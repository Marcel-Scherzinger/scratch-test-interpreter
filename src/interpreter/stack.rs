#[derive(Debug, PartialEq)]
pub enum StackItem<T> {
    Normal(T),
    CountLoop(T, usize),
    PopArgumentFrame(T),
}

impl<T> From<T> for StackItem<T> {
    fn from(value: T) -> Self {
        Self::Normal(value)
    }
}

impl<T> StackItem<T> {
    pub fn value(&self) -> &T {
        match self {
            Self::Normal(t) => t,
            Self::CountLoop(t, _) => t,
            Self::PopArgumentFrame(t) => t,
        }
    }
    pub fn into_value(self) -> T {
        match self {
            Self::Normal(t) => t,
            Self::CountLoop(t, _) => t,
            Self::PopArgumentFrame(t) => t,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Stack<T>(Vec<StackItem<T>>);

impl<T> From<Vec<StackItem<T>>> for Stack<T> {
    fn from(value: Vec<StackItem<T>>) -> Self {
        Self(value)
    }
}

impl<T> Stack<T> {
    pub fn top(&self) -> Option<&StackItem<T>> {
        self.0.last()
    }
    pub fn is_empty(&self) -> bool {
        self.top().is_none()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn pop(&mut self) -> Option<StackItem<T>> {
        self.0.pop()
    }
    pub fn push_opt(&mut self, opt: Option<T>) {
        if let Some(val) = opt {
            self.0.push(val.into());
        }
    }
    pub fn push(&mut self, item: impl Into<StackItem<T>>) {
        self.0.push(item.into());
    }
}
