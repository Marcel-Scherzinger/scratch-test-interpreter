use smodel::{
    attrs::{Expression, dropdowns::OperatorMathop},
    blocks::ExprOrCmpBlockKind,
};
use svalue::SValue;

use crate::{
    State,
    error::{InvalidFileError, LimitError, RResult, RunError, UnsupportedError},
    interpreter::RunningInterpreter,
};

impl<'a, S: State> RunningInterpreter<'a, S> {
    pub(crate) fn eval_expr(
        &mut self,
        expression: &smodel::attrs::Expression,
    ) -> RResult<SValue, S> {
        let block_ref = match expression {
            Expression::Var(var_id) => {
                return Ok(self
                    .state
                    .data_read_variable(var_id)
                    .map_err(RunError::State)?
                    .clone());
            }
            Expression::Lis(lis_id) => {
                return self
                    .state
                    .data_read_textual_list_repr(lis_id)
                    .map_err(RunError::State);
            }
            Expression::Lit(val) => return Ok(val.clone()),
            Expression::Blo(block_ref) => block_ref,
        };
        let block = self
            .doc
            .get_specific_kind(block_ref)
            .map_err(InvalidFileError::NoBlockFoundForReference)?;

        use smodel::blocks::ExprBlockKind as E;

        let block = match block {
            ExprOrCmpBlockKind::Cmp(cmp) => return self.eval_cmp_block_kind(cmp).map(SValue::Bool),
            ExprOrCmpBlockKind::Expr(expr) => expr,
        };

        Ok(match block {
            E::LooksSize | E::LooksCostumenumbername { .. } | E::LooksBackdropnumbername { .. } => {
                return Err(UnsupportedError::UnsupportedLooksBlock.into());
            }
            E::MotionDirection
            | E::MotionXposition
            | E::MotionYposition
            | E::MotionIfonedgebounce => {
                return Err(UnsupportedError::UnsupportedMotionBlock.into());
            }
            E::ArgumentReporterStringNumber { value } => {
                for frame in self.procedure_arguments_frames.iter().rev() {
                    if let Some((_, val)) = frame.argument_for_name(value) {
                        return Ok(val.clone().unwrap_or_else(|| SValue::Text("".into())));
                    }
                }
                return Ok(SValue::Text("".into()));
            }
            E::DataItemoflist { list, index } => {
                let index = self.eval_expr(index)?;
                self.state
                    .data_item_of_list(list, &index)
                    .map_err(RunError::State)?
                    .clone()
            }
            E::DataLengthoflist { list } => self
                .state
                .data_length_of_list(list)
                .map(SValue::Int)
                .map_err(RunError::State)?,
            E::DataItemnumoflist { list, item } => {
                let item = self.eval_expr(item)?;
                self.state
                    .data_item_num_of_list(list, &item)
                    .map(SValue::Int)
                    .map_err(RunError::State)?
            }
            E::OperatorAdd { num1, num2 } => {
                let num1 = self.eval_expr(num1)?;
                let num2 = self.eval_expr(num2)?;
                num1.q_add_numbers(&num2, &mut ())
            }
            E::OperatorMultiply { num1, num2 } => {
                let num1 = self.eval_expr(num1)?;
                let num2 = self.eval_expr(num2)?;
                num1.q_mul_numbers(&num2, &mut ())
            }
            E::OperatorSubtract { num1, num2 } => {
                let num1 = self.eval_expr(num1)?;
                let num2 = self.eval_expr(num2)?;
                num1.q_sub_numbers(&num2, &mut ())
            }
            E::OperatorDivide { num1, num2 } => {
                let num1 = self.eval_expr(num1)?;
                let num2 = self.eval_expr(num2)?;
                num1.q_div_numbers(&num2, &mut ())
            }
            E::OperatorMod { num1, num2 } => {
                let num1 = self.eval_expr(num1)?;
                let num2 = self.eval_expr(num2)?;
                num1.q_modulo(&num2, &mut ()).into()
            }
            E::OperatorMathop { operator, num } => {
                let num = self.eval_expr(num)?;
                match &***operator {
                    OperatorMathop::Ln => num.q_ln(&mut ()),
                    OperatorMathop::Cos => num.q_cos(&mut ()),
                    OperatorMathop::Sin => num.q_sin(&mut ()),
                    OperatorMathop::Exp => num.q_exp(&mut ()),
                    OperatorMathop::Abs => num.q_abs(&mut ()),
                    OperatorMathop::Tan => num.q_tan(&mut ()),
                    OperatorMathop::Log => num.q_log10(&mut ()),
                    OperatorMathop::Sqrt => num.sqrt(&mut ()),
                    OperatorMathop::Floor => num.floor(&mut ()),
                    OperatorMathop::Ceiling => num.ceil(&mut ()),
                    OperatorMathop::ArcusSin => num.q_asin(&mut ()),
                    OperatorMathop::ArcusCos => num.q_acos(&mut ()),
                    OperatorMathop::ArcusTan => num.q_atan(&mut ()),
                    OperatorMathop::TenRaisedTo => num.q_power_of_10(&mut ()),
                }
                .into()
            }
            E::OperatorRound { num } => self.eval_expr(num)?.round(&mut ()).into(),
            E::OperatorRandom { from, to } => {
                let from = self.eval_expr(from)?;
                let to = self.eval_expr(to)?;
                self.state
                    .request_random(from.q_as_number(&mut ())..=to.q_as_number(&mut ()))
                    .map_err(RunError::State)?
                    .into()
            }
            E::OperatorJoin { string1, string2 } => {
                let a = self.eval_expr(string1)?;
                let b = self.eval_expr(string2)?;
                // TODO: think about limits
                a.concat(&b, self.string_max_length)
                    .ok_or(LimitError::StringExceededLengthLimit(
                        self.string_max_length,
                    ))?
                    .into()
            }
            E::OperatorLength { string } => {
                let a = self.eval_expr(string)?;
                SValue::Int(a.textual_length() as i64)
            }
            E::OperatorLetterOf { letter, string } => {
                let letter = self.eval_expr(letter)?;
                let string = self.eval_expr(string)?;
                string.nth_letter_of_me(&letter, &mut ())
            }
            E::SensingAnswer => {
                SValue::Text(self.state.last_answer().map_err(RunError::State)?.into())
            }
            E::SensingTimer
            | E::SensingMousex
            | E::SensingMousey
            | E::SensingDayssince2000
            | E::SensingCurrent { .. }
            | E::SensingDistanceto { .. }
            | E::SensingLoudness
            | E::SensingUsername
            | E::SensingOf { .. }
            | E::SoundVolume => {
                return Err(UnsupportedError::UnsupportedSensingBlock.into());
            }
        })
    }
}
