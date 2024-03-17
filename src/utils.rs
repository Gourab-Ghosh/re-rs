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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct State([Option<(char, usize)>; STATE_ARRAY_SIZE]);

impl State {
    pub fn new(symbol: char, index: usize) -> Self {
        let mut indices_with_symbols = [None; STATE_ARRAY_SIZE];
        indices_with_symbols[0] = Some((symbol, index));
        Self(indices_with_symbols)
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum CustomError {
    InvalidDFAKeys,
    InvalidDFAFinalStates,
    NoDFAStartState,
    InvalidAlphabet,
}
