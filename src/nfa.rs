use super::*;

#[derive(Debug, Default)]
pub struct EpsilonNFA {
    states: HashSet<State>,
    alphabets: HashSet<Option<char>>,
    transition_table: HashMap<(State, Option<char>), HashSet<State>>,
    start_states: HashSet<State>,
    final_states: HashSet<State>,
}

impl EpsilonNFA {
    pub fn to_dfa() -> DFA {
        todo!()
    }
}
