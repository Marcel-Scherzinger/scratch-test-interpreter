use std::collections::HashMap;

use rand::RngExt;
use smodel::attrs::{List, Variable};
use svalue::{SList, SNumber, SValue};

use crate::state::State;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Action {
    Output {
        kind: crate::state::OutputKind,
        message: SValue,
    },
    AskQuestion {
        message: SValue,
    },
    RequestRandom {
        from: SNumber,
        to: SNumber,
        received: SNumber,
    },
    WaitFor {
        secs: SNumber,
    },
}

#[derive(Debug, PartialEq, derive_getters::Getters)]
pub struct DefaultState<'a> {
    variables: HashMap<Variable, SValue>,
    lists: HashMap<List, SList>,
    actions: Vec<Action>,

    answer_index: Option<usize>,
    answer_inputs: Vec<&'a str>,

    randoms: Option<rand::rngs::StdRng>,
}

#[allow(unused)]
impl<'a> DefaultState<'a> {
    pub fn new() -> Self {
        DefaultState {
            variables: HashMap::new(),
            lists: HashMap::new(),
            actions: vec![],
            answer_index: None,
            answer_inputs: vec![],
            randoms: None,
        }
    }
    pub fn set_randoms(&mut self, rng: Option<rand::rngs::StdRng>) -> &mut Self {
        self.randoms = rng;
        self
    }
    pub fn set_answers(&mut self, answers: Vec<&'a str>) -> &mut Self {
        self.answer_inputs = answers;
        self.answer_index = None;
        self
    }
    pub fn variables_mut(&mut self) -> &mut HashMap<Variable, SValue> {
        &mut self.variables
    }
    pub fn lists_mut(&mut self) -> &mut HashMap<List, SList> {
        &mut self.lists
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum DefaultStateError {
    #[error("variable {0:?} does not exist in state")]
    VariableNotFound(Variable),
    #[error("list {0:?} does not exist in state")]
    ListNotFound(List),
    #[error("list {id:?} is full, no item can be inserted")]
    ListFull { id: List },
    #[error("program is not allowed to request random numbers")]
    RandomsDisabled,
    #[error("program asked question without a predefined answer remaining")]
    NoMoreAnswers,
}

impl<'a> State for DefaultState<'a> {
    type Error = DefaultStateError;

    fn ask_question(&mut self, question: SValue) -> Result<(), Self::Error> {
        self.actions.push(Action::AskQuestion { message: question });
        self.answer_index = Some(self.answer_index.map(|x| x + 1).unwrap_or(0));

        self.answer_index
            .and_then(|index| self.answer_inputs.get(index))
            .ok_or(DefaultStateError::NoMoreAnswers)?;
        Ok(())
    }
    fn last_answer(&mut self) -> Result<&str, Self::Error> {
        if let Some(index) = self.answer_index {
            Ok(self
                .answer_inputs
                .get(index)
                .or(self.answer_inputs.last())
                .unwrap_or(&""))
        } else {
            Ok("")
        }
    }

    fn request_int_random(
        &mut self,
        range: std::ops::RangeInclusive<i64>,
    ) -> Result<i64, Self::Error> {
        let received = self
            .randoms
            .as_mut()
            .ok_or(DefaultStateError::RandomsDisabled)?
            .random_range(range.clone());
        let (from, to) = range.into_inner();
        self.actions.push(Action::RequestRandom {
            from: SNumber::Int(from),
            to: SNumber::Int(to),
            received: SNumber::Int(received),
        });
        Ok(received)
    }

    fn request_float_random(
        &mut self,
        range: std::ops::RangeInclusive<f64>,
    ) -> Result<f64, Self::Error> {
        let received = self
            .randoms
            .as_mut()
            .ok_or(DefaultStateError::RandomsDisabled)?
            .random_range(range.clone());
        let (from, to) = range.into_inner();
        self.actions.push(Action::RequestRandom {
            from: SNumber::Float(from),
            to: SNumber::Float(to),
            received: SNumber::Float(received),
        });
        Ok(received)
    }

    fn write_output(&mut self, output_kind: crate::state::OutputKind, message: SValue) {
        self.actions.push(Action::Output {
            kind: output_kind,
            message,
        });
    }

    fn wait_for_secs(&mut self, secs: SNumber) -> Result<(), Self::Error> {
        self.actions.push(Action::WaitFor { secs });
        Ok(())
    }

    fn data_set_variable(&mut self, var_id: &Variable, value: SValue) -> Result<(), Self::Error> {
        let store = self
            .variables
            .get_mut(var_id)
            .ok_or_else(|| unknown_var(var_id))?;
        *store = value;
        Ok(())
    }
    fn data_read_variable(&mut self, var_id: &Variable) -> Result<&SValue, Self::Error> {
        self.variables
            .get(var_id)
            .ok_or_else(|| unknown_var(var_id))
    }
    fn data_insert_at_list(
        &mut self,
        list_id: &List,
        one_based_index: &SValue,
        item: SValue,
    ) -> Result<(), Self::Error> {
        self.list_mut(list_id)?
            .insert_item_at(one_based_index, item, &mut ())
            .map_err(|_err| DefaultStateError::ListFull {
                id: list_id.clone(),
            })?;
        Ok(())
    }
    fn data_replace_list_at(
        &mut self,
        list_id: &List,
        one_based_index: &SValue,
        item: SValue,
    ) -> Result<(), Self::Error> {
        self.list_mut(list_id)?
            .replace_nth_item(one_based_index, item, &mut ());
        Ok(())
    }
    fn data_append_to_list(&mut self, list_id: &List, item: SValue) -> Result<(), Self::Error> {
        self.list_mut(list_id)?
            .append_item(item)
            .map_err(|_err| DefaultStateError::ListFull {
                id: list_id.clone(),
            })?;
        Ok(())
    }
    fn data_list_contains_item(
        &mut self,
        list_id: &List,
        item: &SValue,
    ) -> Result<bool, Self::Error> {
        Ok(self.list_mut(list_id)?.contains_item(item))
    }
    fn data_delete_of_list(
        &mut self,
        list_id: &List,
        one_based_index: &SValue,
    ) -> Result<(), Self::Error> {
        self.list_mut(list_id)?.delete_nth(one_based_index, &mut ());
        Ok(())
    }
    fn data_delete_all_of_list(&mut self, list_id: &List) -> Result<(), Self::Error> {
        self.list_mut(list_id)?.delete_all();
        Ok(())
    }
    fn data_read_textual_list_repr(&mut self, list_id: &List) -> Result<SValue, Self::Error> {
        Ok(self.list(list_id)?.textual_representation().into())
    }
    fn data_item_num_of_list(&mut self, list_id: &List, item: &SValue) -> Result<i64, Self::Error> {
        Ok(self.list(list_id)?.first_index_of_item_in_list(item))
    }
    fn data_item_of_list(
        &mut self,
        list_id: &List,
        one_based_index: &SValue,
    ) -> Result<SValue, Self::Error> {
        Ok(self.list(list_id)?.nth_item(one_based_index, &mut ()))
    }
    fn data_length_of_list(&mut self, list_id: &List) -> Result<i64, Self::Error> {
        Ok(self.list(list_id)?.length())
    }
}

fn unknown_var(var_id: &Variable) -> DefaultStateError {
    DefaultStateError::VariableNotFound(var_id.clone())
}

fn unknown_list(list_id: &List) -> DefaultStateError {
    DefaultStateError::ListNotFound(list_id.clone())
}

impl<'a> DefaultState<'a> {
    #[allow(unused)]
    fn list_mut(&mut self, list_id: &List) -> Result<&mut SList, DefaultStateError> {
        self.lists
            .get_mut(list_id)
            .ok_or_else(|| unknown_list(list_id))
    }
    #[allow(unused)]
    fn list(&mut self, list_id: &List) -> Result<&SList, DefaultStateError> {
        self.lists.get(list_id).ok_or_else(|| unknown_list(list_id))
    }
}
