use smodel::{attrs::RefBlock, blocks::CmpBlockKind};

use crate::{
    State,
    error::{InvalidFileError, RResult, RunError, UnsupportedError},
    interpreter::RunningInterpreter,
};

impl<'a, S: State> RunningInterpreter<'a, S> {
    pub(crate) fn eval_opt_cmp(
        &mut self,
        cmp: &Option<RefBlock<CmpBlockKind>>,
    ) -> RResult<bool, S> {
        let Some(cmp) = cmp else {
            return Ok(false);
        };
        let kind = self
            .doc
            .get_specific_kind(cmp)
            .map_err(InvalidFileError::NoBlockFoundForReference)?;
        self.eval_cmp_block_kind(kind)
    }

    pub(crate) fn eval_cmp_block_kind(&mut self, kind: &CmpBlockKind) -> RResult<bool, S> {
        use smodel::blocks::CmpBlockKind as C;
        // TODO: for now recursive with program stack
        Ok(match kind {
            C::OperatorOr { operand1, operand2 } => {
                let a = self.eval_opt_cmp(operand1)?;
                let b = self.eval_opt_cmp(operand2)?;
                a || b
            }
            C::OperatorAnd { operand1, operand2 } => {
                let a = self.eval_opt_cmp(operand1)?;
                let b = self.eval_opt_cmp(operand2)?;
                a && b
            }
            C::OperatorNot { operand } => {
                let b = self.eval_opt_cmp(operand)?;
                !b
            }
            C::OperatorGt { operand1, operand2 } => {
                let a = self.eval_expr(operand1)?;
                let b = self.eval_expr(operand2)?;
                a > b
            }
            C::OperatorLt { operand1, operand2 } => {
                let a = self.eval_expr(operand1)?;
                let b = self.eval_expr(operand2)?;
                a < b
            }
            C::OperatorEquals { operand1, operand2 } => {
                let a = self.eval_expr(operand1)?;
                let b = self.eval_expr(operand2)?;
                a == b
            }
            C::OperatorContains { string1, string2 } => {
                let a = self.eval_expr(string1)?;
                let b = self.eval_expr(string2)?;
                a.contains_text(&b)
            }
            C::DataListcontainsitem { list, item } => {
                let item = self.eval_expr(item)?;
                self.state
                    .data_list_contains_item(list, &item)
                    .map_err(RunError::State)?
            }
            C::SensingMousedown
            | C::SensingKeyoptions { .. }
            | C::SensingKeypressed { .. }
            | C::SensingTouchingcolor { .. }
            | C::SensingTouchingobject { .. }
            | C::SensingColoristouchingcolor { .. } => {
                return Err(UnsupportedError::UnsupportedSensingBlock.into());
            }
            C::ArgumentReporterBoolean { value } => {
                for frame in self.procedure_arguments_frames.iter().rev() {
                    if let Some((_, v)) = frame.argument_for_name(value) {
                        return Ok(v.as_ref().map(|sv| sv.as_bool()).unwrap_or_default());
                    }
                }
                false // TODO: think about error or keeping this default
            }
        })
    }
}
