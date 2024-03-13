use super::*;

pub trait StringIndentation: ToString {
    fn buffer(&self, size: usize) -> String {
        format!(
            "{}{}{}",
            " ".repeat(size / 2),
            self.to_string(),
            " ".repeat(size - size / 2),
        )
    }

    fn center(&self, size: usize) -> String {
        let self_string = self.to_string();
        self_string.buffer(size.checked_sub(self_string.len()).unwrap_or_default())
    }
}

impl<T: ToString> StringIndentation for T {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug, Hash)]
pub struct State(usize);

impl State {
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    pub fn get_index(self) -> usize {
        self.0
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "q{}", self.get_index())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum CustomError {
    InvalidDFA,
    InvalidAlphabet,
}
