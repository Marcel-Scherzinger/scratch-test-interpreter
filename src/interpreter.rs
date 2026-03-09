mod call_procedures;
mod comparisons;
mod expressions;
mod procedure_frame;
mod stack;
mod statements;
pub use stack::Stack;

use smodel::{
    Id, ProjectDoc,
    blocks::{BlockKind, BlockWrapper},
};
use svalue::ARc;

use crate::{
    State,
    builder::Limits,
    error::{InternalError, InvalidFileError, LimitError, RResult, RunError},
    interpreter::{call_procedures::ProcedureArgumentsFrame, stack::StackItem},
};

#[derive(Debug, PartialEq)]
pub struct RunningInterpreter<'a, S> {
    doc: &'a ProjectDoc,
    execute_stmt_count: usize,
    string_max_length: usize,
    pub(crate) state: S,
    stack: Stack<&'a smodel::Id>,
    procedure_arguments_frames: Vec<ProcedureArgumentsFrame>,
    pub(crate) limits: Limits,
}

impl<'a, S: State> RunningInterpreter<'a, S> {
    pub(crate) fn new(
        limits: Limits,
        doc: &'a ProjectDoc,
        state: S,
        initial_block: &'a smodel::Id,
    ) -> Self {
        Self {
            doc,
            state,
            execute_stmt_count: 0,
            stack: vec![initial_block.into()].into(),
            procedure_arguments_frames: vec![],
            string_max_length: 10 * 1024,
            limits,
        }
    }

    pub(crate) fn internal_start(&mut self) -> RResult<(), S> {
        loop {
            self.execute_stmt_like()?;
            self.execute_stmt_count += 1;
            if self.stack.is_empty() {
                return Ok(());
            }
        }
    }
    fn check_limits(&mut self) -> RResult<(), S> {
        if self
            .limits
            .max_executed_stmts()
            .is_some_and(|limit| self.execute_stmt_count > limit)
        {
            return Err(LimitError::ExceededAllowedStmtCount.into());
        }
        if self
            .limits
            .max_statement_stack_height()
            .is_some_and(|limit| self.stack.len() > limit)
        {
            return Err(LimitError::MaxStackSizeExceeded.into());
        }
        Ok(())
    }

    fn next_block4exec(&mut self) -> RResult<(&'a ARc<BlockWrapper>, StackItem<&'a Id>), S> {
        self.check_limits()?;
        let frame = self
            .stack
            .pop()
            .ok_or(InternalError::PopOnEmptyProgramStack)?;
        match self.doc.get_block(frame.value()) {
            Ok(block) => Ok((block, frame)),
            Err(_) => Err(InvalidFileError::ReachedUnknownBlock(frame.into_value().clone()).into()),
        }
    }

    fn execute_stmt_like(&mut self) -> RResult<(), S> {
        let (block, stack_item) = self.next_block4exec()?;

        match block.inner() {
            BlockKind::Event(event) => {
                self.stack.push_opt(block.next().as_ref());
                self.state
                    .notice_event_block_executed(event, block)
                    .map_err(RunError::State)?;
            }
            BlockKind::ProceduresPrototype(proto) => {
                self.stack.push_opt(block.next().as_ref());
                self.state
                    .notice_proc_proto_block_executed(proto, block)
                    .map_err(RunError::State)?;
            }
            BlockKind::ProceduresDefinition(def) => {
                self.stack.push_opt(block.next().as_ref());
                self.state
                    .notice_proc_def_block_executed(def, block)
                    .map_err(RunError::State)?;
            }
            BlockKind::ExprCmp(_) => Err(InvalidFileError::FoundExpressionWhenStmtExpected(
                block.id().clone(),
            ))?,
            BlockKind::Stmt(stmt) => self.execute_stmt(stmt, block, stack_item)?,
        }
        Ok(())
    }
}
