use smodel::blocks::{BlockWrapper, StmtBlockKind};
use svalue::ARc;

use crate::{
    State,
    error::{InternalError, RResult, RunError, UnsupportedError, UserError},
    interpreter::{RunningInterpreter, stack::StackItem},
    state::OutputKind,
};

impl<'a, S: State> RunningInterpreter<'a, S> {
    pub(crate) fn execute_stmt(
        &mut self,
        stmt: &'a StmtBlockKind,
        block: &'a ARc<BlockWrapper>,
        stack_item: StackItem<&'a smodel::Id>,
    ) -> RResult<(), S> {
        use OutputKind as O;
        use smodel::blocks::StmtBlockKind as K;
        self.state
            .set_statement_as_context(block.id())
            .map_err(RunError::State)?;
        match stmt {
            K::PenUp
            | K::PenDown
            | K::PenClear
            | K::PenStamp
            | K::PenSetPenSizeTo { .. }
            | K::PenChangePenSizeBy { .. }
            | K::PenChangePenColorParamBy { .. }
            | K::PenSetPenColorToColor { .. }
            | K::PenSetPenColorParamTo { .. } => {}
            K::LooksChangeeffectby { .. }
            | K::LooksNextcostume
            | K::LooksNextbackdrop
            | K::LooksSetsizeto { .. }
            | K::LooksSeteffectto { .. }
            | K::LooksCleargraphiceffects
            | K::LooksChangesizeby { .. }
            | K::LooksGoforwardbackwardlayers { .. }
            | K::LooksGotofrontback { .. }
            | K::LooksHide
            | K::LooksShow
            | K::LooksSwitchbackdropto { .. }
            | K::LooksSwitchcostumeto { .. } => {}
            K::MotionPointindirection { .. }
            | K::MotionGoto { .. }
            | K::MotionSetx { .. }
            | K::MotionSety { .. }
            | K::MotionPointtowards { .. }
            | K::MotionGotoxy { .. }
            | K::MotionTurnright { .. }
            | K::MotionTurnleft { .. }
            | K::MotionSetrotationstyle { .. }
            | K::MotionGlideto { .. }
            | K::MotionGlidesecstoxy { .. }
            | K::MotionChangeyby { .. }
            | K::MotionChangexby { .. }
            | K::MotionMovesteps { .. } => {}
            K::SensingSetdragmode { .. } => {}
            K::SoundPlay { .. }
            | K::SoundCleareffects
            | K::SoundStopallsounds
            | K::SoundSeteffectto { .. }
            | K::SoundChangeeffectby { .. }
            | K::SoundChangevolumeby { .. }
            | K::SoundPlayuntildone { .. }
            | K::SoundSetvolumeto { .. } => {}
            K::Text2SpeechSetVoice { .. }
            | K::Text2SpeechSetLanguage { .. }
            | K::Text2SpeechSpeakAndWait { .. } => {}

            K::ControlCreateCloneOf { .. } | K::ControlStartAsClone | K::ControlDeleteThisClone => {
                Err(UnsupportedError::CloneBlocksNotSupported(
                    block.id().clone(),
                ))?;
            }

            K::ControlIf {
                condition,
                substack,
            } => {
                if self.eval_opt_cmp(condition)? {
                    self.stack.push_opt(block.next().as_ref());
                    self.stack.push_opt(substack.as_ref().map(|r| r.id()));
                    return Ok(()); // don't push next again
                }
            }
            K::ControlIfElse {
                condition,
                substack,
                substack2,
            } => {
                self.stack.push_opt(block.next().as_ref());
                if self.eval_opt_cmp(condition)? {
                    self.stack.push_opt(substack.as_ref().map(|r| r.id()));
                } else {
                    self.stack.push_opt(substack2.as_ref().map(|r| r.id()));
                }
                return Ok(()); // don't push next again
            }
            K::ControlRepeatUntil {
                condition,
                substack,
            } => {
                let stop_loop = self.eval_opt_cmp(condition)?;
                if stop_loop {
                    self.stack.push_opt(block.next().as_ref());
                } else if let Some(substack) = substack {
                    self.stack.push(stack_item);
                    self.stack.push(substack.id());
                } else {
                    Err(UserError::ConditionLoopWithoutBodyNeverStops)?;
                }
                return Ok(()); // don't push next again
            }
            K::ControlWaitUntil { condition } => {
                if !self.eval_opt_cmp(condition)? {
                    return Err(UserError::WaitUntilFalseNeverTerminatesInSingleThreadded.into());
                }
            }
            K::ControlWait { duration } => {
                let duration = self.eval_expr(duration)?;
                self.state
                    .wait_for_secs(duration.q_as_number(&mut ()))
                    .map_err(RunError::State)?;
            }
            K::ControlForever { substack } => {
                if let Some(substack) = substack {
                    self.stack.push(stack_item);
                    self.stack.push(substack.id());
                    return Ok(()); // don't push next again
                } else {
                    return Err(UserError::InfiniteLoopWithoutBodyNeverStops.into());
                }
            }
            K::ControlRepeat { times, substack } => {
                let remaining = match stack_item {
                    StackItem::Normal(_) => self
                        .eval_expr(times)?
                        .q_as_number(&mut ())
                        .int_or_border(&mut ())
                        .max(0) as usize,
                    StackItem::CountLoop(_, remaining) => remaining,
                    // in this case the block has to be a procedures call
                    StackItem::PopArgumentFrame(_) => {
                        return Err(InternalError::FatalWrongStackFrameForContext.into());
                    }
                };

                match remaining {
                    0 => self.stack.push_opt(block.next().as_ref()),
                    1.. => {
                        self.stack
                            .push(StackItem::CountLoop(block.id(), remaining - 1));
                        self.stack.push_opt(substack.as_ref().map(|d| d.id()));
                    }
                }
                return Ok(()); // don't push next again
            }
            K::ControlStop { .. } => {
                Err(RunError::TerminatedByControlStop)?;
            }

            K::DataShowlist { .. }
            | K::DataHidelist { .. }
            | K::DataHidevariable { .. }
            | K::DataShowvariable { .. } => {}

            K::DataAddtolist { list, item } => {
                let item = self.eval_expr(item)?;
                self.state
                    .data_append_to_list(list, item)
                    .map_err(RunError::State)?;
            }
            K::DataDeleteoflist { list, index } => {
                let index = self.eval_expr(index)?;
                self.state
                    .data_delete_of_list(list, &index)
                    .map_err(RunError::State)?;
            }
            K::DataInsertatlist { list, index, item } => {
                let index = self.eval_expr(index)?;
                let item = self.eval_expr(item)?;
                self.state
                    .data_insert_at_list(list, &index, item)
                    .map_err(RunError::State)?;
            }
            K::DataDeletealloflist { list } => {
                self.state
                    .data_delete_all_of_list(list)
                    .map_err(RunError::State)?;
            }
            K::DataSetvariableto { variable, value } => {
                let value = self.eval_expr(value)?;
                self.state
                    .data_set_variable(variable, value)
                    .map_err(RunError::State)?;
            }
            K::DataReplaceitemoflist { list, index, item } => {
                let index = self.eval_expr(index)?;
                let item = self.eval_expr(item)?;
                self.state
                    .data_replace_list_at(list, &index, item)
                    .map_err(RunError::State)?;
            }
            K::DataChangevariableby { variable, value } => {
                let value = self.eval_expr(value)?;
                self.state
                    .data_change_variable_by(variable, value)
                    .map_err(RunError::State)?;
            }

            K::EventBroadcast { .. } | K::EventBroadcastandwait { .. } => {
                return Err(UnsupportedError::EventSendingNotSupported.into());
            }

            K::LooksSay { message } => {
                let message = self.eval_expr(message)?;
                self.state.write_output(O::Say, message);
            }
            K::LooksThink { message } => {
                let message = self.eval_expr(message)?;
                self.state.write_output(O::Think, message);
            }
            K::LooksThinkforsecs { message, secs } => {
                let secs = self.eval_expr(secs)?;
                let message = self.eval_expr(message)?;
                self.state
                    .write_output(O::ThinkFor(secs.q_as_float(&mut ())), message);
            }
            K::LooksSayforsecs { message, secs } => {
                let secs = self.eval_expr(secs)?;
                let message = self.eval_expr(message)?;
                self.state
                    .write_output(O::SayFor(secs.q_as_float(&mut ())), message);
            }

            K::SensingAskandwait { question } => {
                let question = self.eval_expr(question)?;
                self.state.ask_question(question).map_err(RunError::State)?;
            }

            K::ProceduresCall {
                procedure_id,
                arguments,
                warp: _,
            } => {
                return self.call_procedure(block, &stack_item, procedure_id, arguments);
            }
        }

        self.stack.push_opt(block.next().as_ref());

        Ok(())
    }
}
