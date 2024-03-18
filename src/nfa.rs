use super::*;

#[derive(Debug, Default, Clone)]
pub struct EpsilonNFA {
    states: HashSet<State>,
    alphabets: HashSet<Option<char>>,
    transition_table: HashMap<(State, Option<char>), HashSet<State>>,
    start_states: HashSet<State>,
    final_states: HashSet<State>,
}

impl EpsilonNFA {
    pub fn new(
        states: HashSet<State>,
        alphabets: HashSet<Option<char>>,
        transition_table: HashMap<(State, Option<char>), HashSet<State>>,
        start_states: HashSet<State>,
        final_states: HashSet<State>,
    ) -> Result<Self, CustomError> {
        let mut nfa = Ok(Self {
            states,
            alphabets,
            transition_table,
            start_states,
            final_states,
        })?;
        if AUTO_OPTIMIZE {
            nfa.minimize();
        }
        Ok(nfa)
    }

    // Simplified function to get adjacent states from a given state
    fn get_adjacent_states(&self, state: &State) -> Vec<State> {
        self.transition_table
            .iter()
            .filter_map(|((from_state, _), to_state)| {
                if from_state == state {
                    Some(to_state)
                } else {
                    None
                }
            })
            .flatten()
            .copied()
            .unique()
            .collect_vec()
    }

    // DFS helper function for topological sorting
    fn dfs(&self, state: State, visited: &mut HashSet<State>, stack: &mut Vec<State>) {
        visited.insert(state);

        // Iterate through adjacent states (assuming we can easily get them from the transition table)
        let adjacent_states = self.get_adjacent_states(&state);
        for adj_state in adjacent_states {
            if !visited.contains(&adj_state) {
                self.dfs(adj_state, visited, stack);
            }
        }

        stack.push(state);
    }

    // Function to perform topological sort
    pub fn get_topologically_sorted(&self) -> Vec<State> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        // Assuming `self.states` is accessible & iterable
        for &state in &self.states {
            if !visited.contains(&state) {
                self.dfs(state, &mut visited, &mut stack);
            }
        }

        stack.reverse(); // Reverse to get the correct order
        stack
    }

    fn remove_unreachable_states_with_custom_start_states_and_transition_table(
        &mut self,
        start_states: &HashSet<State>,
        transition_table: &HashMap<(State, Option<char>), HashSet<State>>,
    ) {
        let mut reachable_states = HashSet::new();
        let mut stack = Vec::new();
        for &state in start_states {
            stack.push(state);
        }
        while let Some(state) = stack.pop() {
            reachable_states.insert(state);
            for &alphabet in &self.alphabets {
                if let Some(next_states) = transition_table.get(&(state, alphabet)) {
                    for &next_state in next_states {
                        if !reachable_states.contains(&next_state) {
                            stack.push(next_state);
                        }
                    }
                }
            }
        }
        self.states.retain(|state| reachable_states.contains(state));
        self.transition_table
            .retain(|(from, _), _| reachable_states.contains(from));
        self.transition_table
            .iter_mut()
            .for_each(|((_, _), to)| to.retain(|state| reachable_states.contains(state)));
        self.start_states
            .retain(|state| reachable_states.contains(state));
        self.final_states
            .retain(|state| reachable_states.contains(state));
    }

    pub fn remove_unreachable_states(&mut self) {
        self.remove_unreachable_states_with_custom_start_states_and_transition_table(
            &self.start_states.clone(),
            &self.transition_table.clone(),
        );
    }

    pub fn remove_trapped_states(&mut self) {
        self.remove_unreachable_states_with_custom_start_states_and_transition_table(
            &self.final_states.clone(),
            &self
                .transition_table
                .iter()
                .flat_map(|(&(from, alphabet), to)| {
                    to.iter().map(move |&state| ((state, alphabet), from))
                })
                .fold(HashMap::new(), |mut hash_map, (key, value)| {
                    hash_map.entry(key).or_insert(HashSet::new()).insert(value);
                    hash_map
                }),
        );
    }

    pub fn minimize(&mut self) {
        self.remove_epsilon_transitions();
        self.remove_trapped_states();
        self.remove_unreachable_states();
    }

    fn epsilon_closure(&self, state: State) -> HashSet<State> {
        let mut closure = HashSet::new();
        let mut stack = vec![state];

        while let Some(s) = stack.pop() {
            if closure.insert(s) {
                if let Some(next_states) = self.transition_table.get(&(s, None)) {
                    for &next_state in next_states {
                        stack.push(next_state);
                    }
                }
            }
        }

        closure
    }

    pub fn remove_epsilon_transitions(&mut self) {
        // TODO: Check Logic
        if AUTO_OPTIMIZE {
            self.remove_unreachable_states()
        }
        let mut new_transition_table = HashMap::new();

        for &state in &self.states {
            let closure = self.epsilon_closure(state);

            for &alphabet in &self.alphabets {
                if let Some(alphabet) = alphabet {
                    let mut new_states = HashSet::new();

                    for &s in &closure {
                        if let Some(next_states) = self.transition_table.get(&(s, Some(alphabet))) {
                            for &next_state in next_states {
                                new_states.extend(self.epsilon_closure(next_state));
                            }
                        }
                    }

                    if !new_states.is_empty() {
                        new_transition_table.insert((state, Some(alphabet)), new_states);
                    }
                }
            }
        }

        self.start_states = self
            .start_states
            .iter()
            .flat_map(|&state| self.epsilon_closure(state))
            .collect();
        self.final_states = self
            .final_states
            .iter()
            .flat_map(|&state| self.epsilon_closure(state))
            .collect();
        self.transition_table = new_transition_table;
        self.alphabets.remove(&None);
        if AUTO_OPTIMIZE {
            self.remove_unreachable_states()
        }
    }

    pub fn get_non_epsilon_nfa(&self) -> Self {
        let mut nfa = self.clone();
        nfa.remove_epsilon_transitions();
        nfa
    }

    pub fn to_dfa(&self) -> DFA {
        let self_copy = self.get_non_epsilon_nfa();
        let mut expand_stack = vec![self_copy
            .start_states
            .iter()
            .copied()
            .sorted_unstable()
            .collect_vec()];
        let mut dfa_states = HashSet::new();
        let dfa_alphabets = self_copy
            .alphabets
            .iter()
            .copied()
            .filter_map(|alphabet| alphabet)
            .collect();
        let mut dfa_transition_table = HashMap::new();
        let epsilon_state = vec![];
        dfa_states.insert(epsilon_state.clone());
        for &alphabet in &dfa_alphabets {
            dfa_transition_table.insert((epsilon_state.clone(), alphabet), epsilon_state.clone());
        }
        while let Some(state) = expand_stack.pop() {
            dfa_states.insert(state.clone());
            for &alphabet in &dfa_alphabets {
                let next_state = state
                    .iter()
                    .copied()
                    .filter_map(|sub_state| {
                        self_copy.transition_table.get(&(sub_state, Some(alphabet)))
                    })
                    .flatten()
                    .copied()
                    .unique()
                    .sorted_unstable()
                    .collect_vec();
                dfa_transition_table.insert((state.clone(), alphabet), next_state.clone());
                if dfa_states.insert(next_state.clone()) {
                    expand_stack.push(next_state.clone());
                }
            }
        }
        let dfa_states_combined = dfa_states
            .iter()
            .map(|v| {
                v.into_iter()
                    .fold(State::new_empty(), |acc, &state| acc.concat(state))
            })
            .collect();
        let dfa_final_states = dfa_states
            .into_iter()
            .filter(|state| {
                state
                    .iter()
                    .copied()
                    .collect::<HashSet<_>>()
                    .intersection(&self_copy.final_states)
                    .next()
                    .is_some()
            })
            .map(|v| {
                v.into_iter()
                    .fold(State::new_empty(), |acc, state| acc.concat(state))
            })
            .collect();
        let mut dfa = DFA::new(
            dfa_states_combined,
            dfa_alphabets,
            dfa_transition_table
                .into_iter()
                .map(|((from, alphabet), to)| {
                    (
                        (
                            from.into_iter()
                                .fold(State::new_empty(), |acc, state| acc.concat(state)),
                            alphabet,
                        ),
                        (to.into_iter()
                            .fold(State::new_empty(), |acc, state| acc.concat(state))),
                    )
                })
                .collect(),
            self_copy
                .start_states
                .iter()
                .sorted_unstable()
                .fold(State::new_empty(), |acc, &state| acc.concat(state)),
            dfa_final_states,
        )
        .unwrap();
        if AUTO_OPTIMIZE {
            dfa.minimize();
        }
        dfa
    }
}

impl From<DFA> for EpsilonNFA {
    fn from(value: DFA) -> Self {
        Self::new(
            value.get_states().clone(),
            value
                .get_alphabets()
                .iter()
                .map(|&alphabet| Some(alphabet))
                .collect(),
            value
                .get_transition_table()
                .iter()
                .map(|(&(from, alphabet), &to)| ((from, Some(alphabet)), HashSet::from([to])))
                .collect(),
            HashSet::from([*value.get_start_state()]),
            value.get_final_states().clone(),
        )
        .unwrap()
    }
}

impl fmt::Display for EpsilonNFA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buffer = 4;
        let states_max_len: usize = self
            .states
            .iter()
            .map(|state| {
                state
                    .get_index()
                    .iter()
                    .map(|(_, index)| 1 + index.to_string().len())
                    .sum()
            })
            .max()
            .unwrap_or_default();
        let separator = "-".repeat(states_max_len.max(4) + states_max_len.max(2) + 3 * buffer + 12);
        let mut transition_table = String::new();
        let topologically_sorted_states_index: HashMap<State, usize> = self
            .get_topologically_sorted()
            .into_iter()
            .enumerate()
            .map(|(index, state)| (state, index))
            .collect();
        let lines = self
            .transition_table
            .iter()
            .sorted_unstable_by_key(|((from, alphabet), _)| {
                (
                    self.start_states.contains(from),
                    topologically_sorted_states_index[from],
                    alphabet,
                )
            })
            .map(|(&(from, alphabet), to)| {
                format!(
                    "|{}|{}|{}|",
                    from.center(states_max_len.max(4) + buffer),
                    alphabet.unwrap_or('ε').center(buffer + 8),
                    format!(
                        "{{{}}}",
                        to.iter()
                            .sorted_unstable()
                            .map(|state| state.to_string())
                            .join(", ")
                    )
                    .center(states_max_len.max(2) + buffer),
                )
            })
            .join(&format!("\n{separator}\n"));
        if lines.is_empty() {
            transition_table.push_str("Empty");
        } else {
            transition_table.push_str(&separator);
            transition_table.push('\n');
            transition_table.push('|');
            transition_table.push_str(&"From".center(states_max_len.max(4) + buffer));
            transition_table.push('|');
            transition_table.push_str(&"Alphabet".center(buffer + 8));
            transition_table.push('|');
            transition_table.push_str(&"To".center(states_max_len.max(2) + buffer));
            transition_table.push('|');
            transition_table.push('\n');
            transition_table.push_str(&separator);
            transition_table.push('\n');
            transition_table.push_str(&lines);
            transition_table.push('\n');
            transition_table.push_str(&separator);
        }
        write!(
            f, "States: {{{}}}\nAlphabets: {{{}}}\nStart States: {{{}}}\nFinal States: {{{}}}\n\nTransition Table:\n\n{}",
            self.states.iter().sorted_unstable().map(|state| state.to_string()).join(", "),
            self.alphabets.iter().sorted_unstable().map(|alphabet| alphabet.unwrap_or('ε')).join(", "),
            self.start_states.iter().sorted_unstable().map(|state| state.to_string()).join(", "),
            self.final_states.iter().sorted_unstable().map(|state| state.to_string()).join(", "),
            transition_table,
        )
    }
}
