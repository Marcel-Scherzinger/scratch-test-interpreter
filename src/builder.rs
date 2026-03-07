mod limits;
pub use limits::Limits;

use smodel::ProjectDoc;

use crate::{State, error::RunError, interpreter::RunningInterpreter};

#[derive(Debug, PartialEq, Clone)]
pub struct Interpreter {
    limits: Limits,
}

#[derive(Debug, derive_getters::Getters)]
#[allow(unused)]
pub struct Report<'a, S: State> {
    doc: &'a ProjectDoc,
    state: S,
    error_code: Option<RunError<S::Error>>,
    limits: Limits,
}
#[allow(unused)]
impl Interpreter {
    pub fn new_restrictive() -> Self {
        Self {
            limits: Limits::RESTRICTIVE,
        }
    }
    pub fn run<'a, S: State>(
        self,
        doc: &'a ProjectDoc,
        state: S,
        initial_block: &'a smodel::Id,
    ) -> Report<'a, S> {
        let mut running = RunningInterpreter::new(self.limits, doc, state, initial_block);
        let error_code = running.internal_start().err();
        Report {
            doc,
            state: running.state,
            error_code,
            limits: running.limits,
        }
    }
    pub const fn limits_mut(&mut self) -> &mut Limits {
        &mut self.limits
    }
    pub const fn limits(&mut self) -> &Limits {
        &self.limits
    }
    pub const fn set_limits(&mut self, limits: Limits) -> &mut Self {
        self.limits = limits;
        self
    }
    pub const fn with_limits(mut self, limits: Limits) -> Self {
        self.limits = limits;
        self
    }
    pub fn change_limits(mut self, f: impl FnOnce(Limits) -> Limits) -> Self {
        self.limits = f(self.limits);
        self
    }
}
