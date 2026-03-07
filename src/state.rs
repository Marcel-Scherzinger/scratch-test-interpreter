use smodel::{
    attrs::{List, Variable},
    blocks::{BlockWrapper, EventBlockKind, ProceduresDefinition, ProceduresPrototype},
};
use std::ops::RangeInclusive;
use svalue::{ARc, SNumber, SValue};

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum OutputKind {
    Think,
    ThinkFor(f64),
    Say,
    SayFor(f64),
}

pub trait State {
    type Error;

    fn set_statement_as_context(&mut self, _id: &smodel::Id) -> Result<(), Self::Error> {
        Ok(())
    }

    fn ask_question(&mut self, question: SValue) -> Result<(), Self::Error>;
    fn last_answer(&mut self) -> Result<&str, Self::Error>;
    fn request_random(
        &mut self,
        range: RangeInclusive<SNumber>,
    ) -> Result<svalue::SNumber, Self::Error> {
        let (lower, upper) = range.into_inner();
        match (lower, upper) {
            (SNumber::Float(_), _) | (_, SNumber::Float(_)) => self
                .request_float_random(lower.q_as_float(&mut ())..=upper.q_as_float(&mut ()))
                .map(SNumber::Float),
            (SNumber::Int(lower), SNumber::Int(upper)) => {
                self.request_int_random(lower..=upper).map(SNumber::Int)
            }
        }
    }

    fn request_float_random(&mut self, range: RangeInclusive<f64>) -> Result<f64, Self::Error>;
    fn request_int_random(&mut self, range: RangeInclusive<i64>) -> Result<i64, Self::Error>;

    fn data_set_variable(&mut self, var_id: &Variable, value: SValue) -> Result<(), Self::Error>;
    fn data_change_variable_by(
        &mut self,
        var_id: &Variable,
        by: SValue,
    ) -> Result<(), Self::Error> {
        let new = self.data_read_variable(var_id)?.q_add_numbers(&by, &mut ());
        self.data_set_variable(var_id, new)
    }
    fn data_read_variable(&mut self, var_id: &Variable) -> Result<&SValue, Self::Error>;
    fn write_output(&mut self, output_kind: OutputKind, message: SValue);
    fn wait_for_secs(&mut self, _secs: SNumber) -> Result<(), Self::Error> {
        Ok(())
    }

    fn data_list_contains_item(
        &mut self,
        list_id: &List,
        item: &SValue,
    ) -> Result<bool, Self::Error>;
    fn data_read_textual_list_repr(&mut self, list_id: &List) -> Result<SValue, Self::Error>;
    fn data_append_to_list(&mut self, list_id: &List, item: SValue) -> Result<(), Self::Error>;
    fn data_item_of_list(
        &mut self,
        list_id: &List,
        one_based_index: &SValue,
    ) -> Result<SValue, Self::Error>;
    fn data_item_num_of_list(&mut self, list_id: &List, item: &SValue) -> Result<i64, Self::Error>;
    fn data_length_of_list(&mut self, list_id: &List) -> Result<i64, Self::Error>;
    fn data_delete_of_list(
        &mut self,
        list_id: &List,
        one_based_index: &SValue,
    ) -> Result<(), Self::Error>;
    fn data_delete_all_of_list(&mut self, list_id: &List) -> Result<(), Self::Error>;
    fn data_insert_at_list(
        &mut self,
        list_id: &List,
        one_based_index: &SValue,
        item: SValue,
    ) -> Result<(), Self::Error>;
    fn data_replace_list_at(
        &mut self,
        list_id: &List,
        one_based_index: &SValue,
        item: SValue,
    ) -> Result<(), Self::Error>;

    fn notice_event_block_executed(
        &mut self,
        _event: &EventBlockKind,
        _block: &ARc<BlockWrapper>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn notice_proc_def_block_executed(
        &mut self,
        _def: &ProceduresDefinition,
        _block: &ARc<BlockWrapper>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn notice_proc_proto_block_executed(
        &mut self,
        _proto: &ProceduresPrototype,
        _block: &ARc<BlockWrapper>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
