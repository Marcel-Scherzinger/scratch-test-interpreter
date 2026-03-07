use std::collections::HashMap;

use smodel::{
    attrs::ProcedureArgumentId,
    blocks::{ProcedureArgumentDef, ProcedureId},
};
use svalue::ARc;

use smodel::{attrs::Expression, blocks::BlockWrapper};
use svalue::SValue;

use crate::{
    State,
    error::{InternalError, InvalidFileError, RResult, RunError},
    interpreter::{RunningInterpreter, stack::StackItem},
};

impl<'a, S: State> RunningInterpreter<'a, S> {
    pub(crate) fn call_procedure(
        &mut self,
        block: &'a ARc<BlockWrapper>,
        stack_item: &StackItem<&'a smodel::Id>,
        procedure_id: &'a ProcedureId,
        arguments: &HashMap<ProcedureArgumentId, Option<Expression>>,
    ) -> RResult<(), S> {
        match stack_item {
            // call will be initiated
            StackItem::Normal(_) => {
                let evaluated_arguments: Result<
                    HashMap<&ProcedureArgumentId, Option<SValue>>,
                    RunError<S::Error>,
                > = arguments
                    .iter()
                    .map(|(id, expr)| {
                        Ok((id, expr.as_ref().map(|e| self.eval_expr(e)).transpose()?))
                    })
                    .collect();

                let procedure = self
                    .doc
                    .targets()
                    .iter()
                    .flat_map(|t| t.procedures().get(procedure_id))
                    .next()
                    .ok_or(InvalidFileError::CantFindProcedureForCall)?;

                let mut evaluated_arguments = evaluated_arguments?;
                let mut arguments_by_name = HashMap::new();
                for arg in procedure.arguments().iter() {
                    if let Some(value) = evaluated_arguments.remove(arg.argument_id()) {
                        arguments_by_name.insert(arg.name().clone(), (arg.clone(), value));
                    } else {
                        Err(InvalidFileError::MissingArgForProcedureParameter)?;
                    }
                }
                if !evaluated_arguments.is_empty() {
                    Err(InvalidFileError::ExtraArgWithoutProcedureParameter)?;
                }

                let frame = ProcedureArgumentsFrame {
                    procedure_id: procedure.procedure_id().clone(),
                    arguments_by_name,
                };
                // log::trace!("Add procedure arguments frame: {frame:?}");
                let definition_block_id = procedure.definition_block().id();

                self.stack.push_opt(block.next().as_ref());
                // IMPORTANT: prepare cleanup on stack
                self.stack.push(StackItem::PopArgumentFrame(block.id()));
                self.procedure_arguments_frames.push(frame);
                self.stack.push(definition_block_id);
                // log::debug!("[{scope}] call procedure: {proccode:?}");
            }
            // call will be cleaned up (it is now after the exit of the call)
            StackItem::PopArgumentFrame(_) => {
                self.procedure_arguments_frames
                    .pop()
                    .ok_or(InternalError::PopOnEmptyArgumentFramesStack)?;
            }
            // only possible for loops
            StackItem::CountLoop(_, _) => {
                Err(InternalError::FatalWrongStackFrameForContext)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct ProcedureArgumentsFrame {
    procedure_id: ProcedureId,
    arguments_by_name: HashMap<String, (ProcedureArgumentDef, Option<svalue::SValue>)>,
}

impl ProcedureArgumentsFrame {
    pub fn argument_for_name(
        &self,
        name: &str,
    ) -> Option<&(ProcedureArgumentDef, Option<svalue::SValue>)> {
        self.arguments_by_name.get(name)
    }
}
