use super::*;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
pub struct DFA {
    states: HashSet<State>,
    alphabets: HashSet<char>,
    transition_table: HashMap<(State, char), State>,
    start_state: State,
    current_state: State,
    final_states: HashSet<State>,
    trapped_states: HashSet<State>,
}

impl DFA {
    pub fn new_unoptimized(
        states: HashSet<State>,
        alphabets: HashSet<char>,
        transition_table: HashMap<(State, char), State>,
        start_state: State,
        final_states: HashSet<State>,
    ) -> Result<Self, CustomError> {
        let mut dfa = Self {
            states,
            alphabets,
            transition_table,
            start_state,
            current_state: start_state,
            final_states,
            trapped_states: HashSet::default(),
        };
        dfa.check_validity()?;
        dfa.update_trapped_states();
        Ok(dfa)
    }

    pub fn new(
        states: HashSet<State>,
        alphabets: HashSet<char>,
        transition_table: HashMap<(State, char), State>,
        start_state: State,
        final_states: HashSet<State>,
    ) -> Result<Self, CustomError> {
        let mut dfa = Self::new_unoptimized(
            states,
            alphabets,
            transition_table,
            start_state,
            final_states,
        )?;
        if AUTO_OPTIMIZE {
            dfa.minimize();
        }
        Ok(dfa)
    }

    pub fn update_trapped_states(&mut self) {
        let mut reachable_states = HashSet::new();
        let mut stack = Vec::new();
        for &state in &self.final_states {
            reachable_states.insert(state);
            stack.push(state);
        }
        let mut reversed_transition_table: HashMap<(State, char), HashSet<State>> = HashMap::new();
        for (&(from, alphabet), &to) in &self.transition_table {
            reversed_transition_table
                .entry((to, alphabet))
                .or_default()
                .insert(from);
        }
        while let Some(state) = stack.pop() {
            for &alphabet in &self.alphabets {
                if let Some(prev_states) = reversed_transition_table.get(&(state, alphabet)) {
                    for &prev_state in prev_states {
                        if !reachable_states.contains(&prev_state) {
                            reachable_states.insert(prev_state);
                            stack.push(prev_state);
                        }
                    }
                }
            }
        }
        self.trapped_states = self.states.difference(&reachable_states).cloned().collect();
    }

    pub fn remove_unreachable_states(&mut self) {
        let mut reachable_states = HashSet::new();
        let mut stack = Vec::new();
        stack.push(self.start_state);

        while let Some(state) = stack.pop() {
            reachable_states.insert(state);
            for &alphabet in &self.alphabets {
                if let Some(&next_state) = self.transition_table.get(&(state, alphabet)) {
                    if !reachable_states.contains(&next_state) {
                        stack.push(next_state);
                    }
                }
            }
        }

        self.states.retain(|state| reachable_states.contains(state));
        self.transition_table.retain(|(from, _), to| {
            reachable_states.contains(from) && reachable_states.contains(to)
        });
        self.final_states
            .retain(|state| reachable_states.contains(state));
        self.trapped_states
            .retain(|state| reachable_states.contains(state));
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

    // Simplified function to get adjacent states from a given state
    fn get_adjacent_states(&self, state: &State) -> Vec<State> {
        self.transition_table
            .iter()
            .filter_map(|((from_state, _), &to_state)| {
                if from_state == state {
                    Some(to_state)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn rename_states(&mut self) {
        let topologically_sorted_states_index: HashMap<State, usize> = self
            .get_topologically_sorted()
            .into_iter()
            .enumerate()
            .map(|(index, state)| (state, index))
            .collect();
        let states_mapping = self
            .states
            .iter()
            .sorted_unstable_by_key(|&state| {
                (
                    state != &self.start_state,
                    self.trapped_states.contains(state),
                    topologically_sorted_states_index[state],
                )
            })
            .enumerate()
            .map(|(index, &state)| (state, State::new(DEFAULT_STATE_SYMBOL, index)))
            .collect::<HashMap<_, _>>();
        self.states = self
            .states
            .iter()
            .map(|state| states_mapping[state])
            .collect();
        self.transition_table = self
            .transition_table
            .iter()
            .map(|(&(from, alphabet), &to)| {
                ((states_mapping[&from], alphabet), states_mapping[&to])
            })
            .collect();
        self.start_state = states_mapping[&self.start_state];
        self.final_states = self
            .final_states
            .iter()
            .map(|&state| states_mapping[&state])
            .collect();
        self.trapped_states = self
            .trapped_states
            .iter()
            .map(|&state| states_mapping[&state])
            .collect();
    }

    pub fn add_state(&mut self, state: State) {
        self.states.insert(state);
    }

    pub fn add_alphabet(&mut self, alphabet: char) {
        self.alphabets.insert(alphabet);
    }

    pub fn add_transition(&mut self, from: State, to: State, alphabet: char) {
        self.transition_table.insert((from, alphabet), to);
    }

    pub fn set_start_state(&mut self, state: State) {
        self.start_state = state;
    }

    pub fn set_current_state(&mut self, state: State) {
        self.current_state = state;
    }

    pub fn add_final_state(&mut self, state: State) {
        self.final_states.insert(state);
    }

    pub fn update_to_next_state(&mut self, alphabet: char) -> Result<(), CustomError> {
        // let curr_state = self.current_state;
        self.current_state = *self
            .transition_table
            .get(&(self.current_state, alphabet))
            .ok_or(CustomError::InvalidAlphabet)?;
        // println!(
        //     "State changed from {} to {}",
        //     curr_state.get_index(),
        //     self.current_state.get_index()
        // );
        Ok(())
    }

    pub fn check_validity(&self) -> Result<(), CustomError> {
        let mut expected_keys = self
            .states
            .iter()
            .copied()
            .cartesian_product(self.alphabets.iter().copied())
            .collect_vec();
        let mut actual_keys = self.transition_table.keys().copied().collect_vec();
        expected_keys.sort();
        actual_keys.sort();
        if expected_keys != actual_keys {
            return Err(CustomError::InvalidDFAKeys);
        }
        if !self.states.contains(&self.start_state) {
            return Err(CustomError::NoDFAStartState);
        }
        if !IS_TESTING && !self.states.is_superset(&self.final_states) {
            return Err(CustomError::InvalidDFAFinalStates);
        }
        Ok(())
    }

    pub fn minimize(&mut self) {
        self.remove_unreachable_states();

        let mut partition = vec![
            self.final_states.clone(),
            self.states
                .difference(&self.final_states)
                .cloned()
                .collect(),
        ];
        let mut refined = true;

        while refined {
            refined = false;
            let mut new_partition = Vec::new();

            for group in &partition {
                let mut split_sets: Vec<HashSet<State>> = Vec::new();

                for &state in group {
                    if let Some(index) = split_sets.iter().position(|set| {
                        self.alphabets.iter().all(|&ch| {
                            let set_representative = *set.iter().next().unwrap();
                            let next_state = self.transition_table[&(set_representative, ch)];
                            partition.iter().any(|group| {
                                group.contains(&self.transition_table[&(state, ch)])
                                    && group.contains(&next_state)
                            })
                        })
                    }) {
                        split_sets[index].insert(state);
                    } else {
                        let mut new_set = HashSet::new();
                        new_set.insert(state);
                        split_sets.push(new_set);
                    }
                }

                refined |= split_sets.len() > 1;
                for set in split_sets {
                    new_partition.push(set);
                }
            }
            partition = new_partition;
        }

        let mut new_states = HashSet::new();
        let mut new_transition_table = HashMap::new();
        let mut new_final_states = HashSet::new();

        let group_representative: HashMap<State, State> =
            HashMap::from_iter(partition.iter().flat_map(|group| {
                group
                    .iter()
                    .map(|&state| (state, *group.iter().min().unwrap()))
            }));

        for group in &partition {
            let representative_state = group_representative[group.iter().next().unwrap()];
            new_states.insert(representative_state);

            if group.contains(&self.start_state) {
                self.start_state = representative_state;
            }

            if group.intersection(&self.final_states).count() > 0 {
                new_final_states.insert(representative_state);
            }

            for &alphabet in &self.alphabets {
                let next_state = self.transition_table[&(representative_state, alphabet)];
                let partition_index = partition
                    .iter()
                    .position(|s| s.contains(&next_state))
                    .unwrap();
                let partition_state =
                    group_representative[partition[partition_index].iter().min().unwrap()];
                new_transition_table.insert((representative_state, alphabet), partition_state);
            }
        }

        self.states = new_states;
        self.transition_table = new_transition_table;
        self.final_states = new_final_states;
        self.update_trapped_states();
        self.rename_states();
    }

    pub fn get_minimized(&self) -> Self {
        let mut dfa = self.clone();
        dfa.minimize();
        dfa
    }

    fn product<'a>(
        &self,
        other: &DFA,
        possible_final_states: impl Iterator<Item = (State, State)>,
    ) -> Result<Self, CustomError> {
        let mut expand_stack = vec![(self.start_state, other.start_state)];
        let mut new_states = HashSet::new();
        let new_alphabets: HashSet<char> =
            HashSet::from_iter(self.alphabets.union(&other.alphabets).copied());
        let mut new_transition_table = HashMap::new();
        let trapped_state = (State::new('T', 0), State::new('T', 0));
        new_states.insert(trapped_state);
        for &alphabet in &new_alphabets {
            new_transition_table.insert((trapped_state, alphabet), trapped_state);
        }
        while let Some(state) = expand_stack.pop() {
            new_states.insert(state);
            for &alphabet in &new_alphabets {
                let next_state = if let (Some(&state1), Some(&state2)) = (
                    self.transition_table.get(&(state.0, alphabet)),
                    other.transition_table.get(&(state.1, alphabet)),
                ) {
                    (state1, state2)
                } else {
                    trapped_state
                };
                new_transition_table.insert((state, alphabet), next_state);
                if new_states.insert(next_state) {
                    expand_stack.push(next_state);
                }
            }
        }
        let new_states_combined: HashSet<State> = new_states
            .into_iter()
            .map(|(state1, state2)| state1.concat(state2))
            .collect();
        let new_final_states = possible_final_states
            .filter_map(|(state1, state2)| {
                let state = state1.concat(state2);
                if new_states_combined.contains(&state) {
                    Some(state)
                } else {
                    None
                }
            })
            .collect();
        Self::new(
            new_states_combined,
            new_alphabets,
            new_transition_table
                .into_iter()
                .map(|((from, alphabet), to)| {
                    ((from.0.concat(from.1), alphabet), (to.0.concat(to.1)))
                })
                .collect(),
            self.start_state.concat(other.start_state),
            new_final_states,
        )
    }

    pub fn intersection(&self, other: &DFA) -> Result<Self, CustomError> {
        self.product(
            other,
            self.final_states
                .iter()
                .copied()
                .cartesian_product(other.final_states.iter().copied()),
        )
    }

    pub fn union(&self, other: &DFA) -> Result<Self, CustomError> {
        self.product(
            other,
            self.final_states
                .iter()
                .copied()
                .cartesian_product(other.states.iter().copied())
                .chain(
                    self.states
                        .iter()
                        .copied()
                        .cartesian_product(other.final_states.iter().copied()),
                ),
        )
    }

    pub fn difference(&self, other: &DFA) -> Result<Self, CustomError> {
        self.product(
            other,
            self.final_states
                .iter()
                .copied()
                .cartesian_product(other.states.difference(&other.final_states).copied()),
        )
    }

    pub fn complement(&mut self) {
        self.final_states = self
            .states
            .difference(&self.final_states)
            .copied()
            .collect();
        self.update_trapped_states();
    }

    pub fn get_complement(&self) -> Self {
        let mut dfa_copy = self.clone();
        dfa_copy.complement();
        dfa_copy
    }

    pub fn accepts(&mut self, text: &str) -> Result<bool, CustomError> {
        self.current_state = self.start_state;
        for ch in text.chars() {
            self.update_to_next_state(ch)?;
        }
        Ok(self.final_states.contains(&self.current_state))
    }

    pub fn find(&mut self, text: &str) -> Result<Option<Range<usize>>, CustomError> {
        self.current_state = self.start_state;
        let mut start = 0;
        let mut end = 0;
        let mut match_found = self.final_states.contains(&self.current_state);

        for (i, ch) in text.char_indices() {
            if self.trapped_states.contains(&self.current_state) {
                self.current_state = self.start_state;
                start = i;
                end = i;
                match_found = false;
            }
            self.update_to_next_state(ch)?;
            if self.final_states.contains(&self.current_state) {
                match_found = true;
            }
            if match_found && !self.final_states.contains(&self.current_state) {
                break;
            }
            if !self.trapped_states.contains(&self.current_state) {
                end += ch.len_utf8();
            }
        }
        if match_found {
            // println!("Matched {:?}", start..end);
            return Ok(Some(start..end));
        }
        Ok(None)
    }

    pub fn find_all<'a>(
        &'a mut self,
        text: &'a str,
    ) -> Result<impl Iterator<Item = (Range<usize>, &'a str)>, CustomError> {
        if text.chars().unique().collect::<HashSet<_>>() != self.alphabets {
            return Err(CustomError::InvalidAlphabet);
        }
        Ok(MatchIterator {
            dfa: self,
            text,
            start: 0,
        }
        .map(|range| (range.clone(), &text[range])))
    }
}

impl fmt::Display for DFA {
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
                    from != &self.start_state,
                    self.trapped_states.contains(from),
                    topologically_sorted_states_index[from],
                    alphabet,
                )
            })
            .map(|(&(from, alphabet), &to)| {
                format!(
                    "|{}|{}|{}|",
                    from.center(states_max_len.max(4) + buffer),
                    alphabet.center(buffer + 8),
                    to.center(states_max_len.max(2) + buffer),
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
            f, "States: [{}]\nAlphabets: [{}]\nStart State: {}\nFinal States: [{}]\nTrapped States: [{}]\n\nTransition Table:\n\n{}",
            self.states.iter().sorted().map(|state| state.to_string()).join(", "),
            self.alphabets.iter().sorted().join(", "),
            self.start_state,
            self.final_states.iter().sorted().map(|state| state.to_string()).join(", "),
            self.trapped_states.iter().sorted().map(|state| state.to_string()).join(", "),
            transition_table,
        )
    }
}
pub struct MatchIterator<'a> {
    dfa: &'a mut DFA,
    text: &'a str,
    start: usize,
}

impl<'a> Iterator for MatchIterator<'a> {
    type Item = Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.text.len() {
            return None;
        }
        if let Some(mut slice_index) = self.dfa.find(&self.text[self.start..]).unwrap() {
            slice_index.start += self.start;
            slice_index.end += self.start;
            self.start = slice_index.end.max(self.start + 1);
            return Some(slice_index);
        }
        None
    }
}
