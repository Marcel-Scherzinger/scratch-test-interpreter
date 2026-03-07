#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Limits {
    max_string_length: Option<usize>,
    max_executed_stmts: Option<usize>,
    max_statement_stack_height: Option<usize>,
}

#[allow(unused)]
impl Limits {
    pub const RESTRICTIVE: Limits = Limits {
        max_statement_stack_height: Some(100),
        max_string_length: Some(1024),
        max_executed_stmts: Some(10 * 1024),
    };
    pub const fn new() -> Self {
        Self {
            max_string_length: None,
            max_executed_stmts: None,
            max_statement_stack_height: None,
        }
    }

    pub const fn max_string_length(&self) -> &Option<usize> {
        &self.max_string_length
    }
    pub const fn max_executed_stmts(&self) -> &Option<usize> {
        &self.max_executed_stmts
    }
    pub const fn max_statement_stack_height(&self) -> &Option<usize> {
        &self.max_statement_stack_height
    }

    pub const fn set_max_string_length(&mut self, len: Option<usize>) -> &mut Self {
        self.max_string_length = len;
        self
    }
    pub const fn set_max_statement_stack_height(&mut self, v: Option<usize>) -> &mut Self {
        self.max_statement_stack_height = v;
        self
    }
    pub const fn set_max_executed_stmts(&mut self, v: Option<usize>) -> &mut Self {
        self.max_executed_stmts = v;
        self
    }

    pub const fn with_max_string_length(mut self, len: Option<usize>) -> Self {
        self.set_max_string_length(len);
        self
    }
    pub const fn with_max_statement_stack_height(mut self, v: Option<usize>) -> Self {
        self.set_max_statement_stack_height(v);
        self
    }
    pub const fn with_max_executed_stmts(mut self, v: Option<usize>) -> Self {
        self.set_max_executed_stmts(v);
        self
    }
}
