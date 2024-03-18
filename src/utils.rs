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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct State([Option<(char, usize)>; STATE_ARRAY_SIZE]);

impl State {
    pub fn new_empty() -> Self {
        Self([None; STATE_ARRAY_SIZE])
    }

    pub fn new(symbol: char, index: usize) -> Self {
        let mut state = Self::new_empty();
        state.0[0] = Some((symbol, index));
        state
    }

    pub fn get_index(self) -> Vec<(char, usize)> {
        self.0.iter().filter_map(|&index| index).collect_vec()
    }

    pub fn concat(self, other: Self) -> Self {
        let mut index = self.0;
        index
            .iter_mut()
            .filter(|state| state.is_none())
            .zip(other.0.iter().filter(|state| state.is_some()))
            .for_each(|(name, &new_name)| *name = new_name);
        Self(index)
    }
}

impl From<State> for usize {
    fn from(value: State) -> Self {
        assert!(
            value.get_index().len() < 2,
            "Cannot convert {value} to usize"
        );
        value
            .get_index()
            .get(0)
            .map(|(_, index)| *index)
            .unwrap_or(usize::MAX)
    }
}

impl From<usize> for State {
    fn from(value: usize) -> Self {
        Self::new('q', value)
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state_string = self
            .get_index()
            .into_iter()
            .map(|(symbol, index)| format!("{symbol}{index}"))
            .join("");
        if state_string.is_empty() {
            state_string += "epsilon";
        }
        write!(f, "{state_string}")
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum CustomError {
    InvalidDFAKeys,
    InvalidDFAFinalStates,
    NoDFAStartState,
    InvalidAlphabet,
}
