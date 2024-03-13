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
        if !dfa.is_valid() {
            return Err(CustomError::InvalidDFA);
        }
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
        self.transition_table.iter()
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
        let topologically_sorted_states_index: HashMap<State, usize> = self.get_topologically_sorted().into_iter().enumerate().map(|(index, state)| (state, index)).collect();
        let states_mapping = self
            .states
            .iter()
            .sorted_unstable_by_key(|&state| {
                (
                    state != &self.start_state,
                    !self.trapped_states.contains(state),
                    topologically_sorted_states_index[state],
                )
            })
            .enumerate()
            .map(|(index, &state)| (state, State::new(index)))
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

    pub fn is_valid(&self) -> bool {
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
            return false;
        }
        if !self.states.contains(&self.start_state) {
            return false;
        }
        if !self.states.is_superset(&self.final_states) {
            return false;
        }
        true
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

        let group_representative: HashMap<State, State> = HashMap::from_iter(
            partition
                .iter()
                .flat_map(|group| {
                    group
                        .iter()
                        .map(|&state| (state, *group.iter().min().unwrap()))
                }),
        );

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

    fn map_state_index(&self, state1: State, state2: State) -> usize {
        state1.get_index() + self.states.len() * state2.get_index()
    }

    fn map_state(&self, state1: State, state2: State) -> State {
        State::new(self.map_state_index(state1, state2))
    }

    pub fn intersection(&self, other: &DFA) -> Result<Self, CustomError> {
        let mut self_copy = self.clone();
        let mut other_copy = other.clone();
        if AUTO_OPTIMIZE {
            self_copy.minimize();
            other_copy.minimize();
        }
        self_copy.rename_states();
        other_copy.rename_states();
        let alphabets = self_copy
            .alphabets
            .union(&other_copy.alphabets)
            .copied()
            .collect::<HashSet<_>>();
        let trapped_state = State::new(self_copy.states.len() * other_copy.states.len());
        let mut states = HashSet::default();
        states.insert(trapped_state);
        let mut transition_table = HashMap::default();
        for (&state1, &state2) in self_copy
            .states
            .iter()
            .cartesian_product(other_copy.states.iter())
        {
            let new_state = self_copy.map_state(state1, state2);
            states.insert(new_state);
            for &alphabet in &alphabets {
                let mapped_state1 = self_copy
                    .transition_table
                    .get(&(state1, alphabet))
                    .unwrap_or(&trapped_state);
                let mapped_state2 = other_copy
                    .transition_table
                    .get(&(state2, alphabet))
                    .unwrap_or(&trapped_state);
                transition_table.insert(
                    (new_state, alphabet),
                    self_copy.map_state(*mapped_state1, *mapped_state2),
                );
            }
        }
        for &alphabet in &alphabets {
            transition_table.insert((trapped_state, alphabet), trapped_state);
        }
        let mut dfa_intersection = Self::new_unoptimized(
            states,
            alphabets,
            transition_table,
            State::new(
                self_copy.start_state.get_index()
                    + self_copy.states.len() * other_copy.start_state.get_index(),
            ),
            HashSet::default(),
        )?;
        dfa_intersection.final_states = self_copy
            .states
            .iter()
            .cartesian_product(other_copy.states.iter())
            .filter(|(state1, state2)| {
                self_copy.final_states.contains(state1) || other_copy.final_states.contains(state2)
            })
            .map(|(state1, state2)| self_copy.map_state(*state1, *state2))
            .collect();
        if AUTO_OPTIMIZE {
            dfa_intersection.minimize();
        } else {
            dfa_intersection.remove_unreachable_states();
            dfa_intersection.update_trapped_states();
            dfa_intersection.rename_states();
        }
        Ok(dfa_intersection)
    }

    pub fn union(&self, other: &DFA) -> Result<Self, CustomError> {
        self.get_complement()
            .intersection(&other.get_complement())
            .map(|dfa| dfa.get_complement())
    }

    pub fn difference(&self, other: &DFA) -> Result<Self, CustomError> {
        self.intersection(&other.get_complement())
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
        let states_len = self
            .states
            .iter()
            .map(|state| state.get_index())
            .max()
            .unwrap_or_default()
            .to_string()
            .len()
            + 1;
        let separator = "-".repeat(states_len.max(4) + states_len.max(2) + 3 * buffer + 12);
        let mut transition_table = String::new();
        let lines = self
            .transition_table
            .iter()
            .sorted()
            .map(|(&(from, alphabet), &to)| {
                format!(
                    "|{}|{}|{}|",
                    from.center(states_len.max(4) + buffer),
                    alphabet.center(buffer + 8),
                    to.center(states_len.max(2) + buffer),
                )
            })
            .join(&format!("\n{separator}\n"));
        if lines.is_empty() {
            transition_table.push_str("Empty");
        } else {
            transition_table.push_str(&separator);
            transition_table.push('\n');
            transition_table.push('|');
            transition_table.push_str(&"From".center(states_len.max(4) + buffer));
            transition_table.push('|');
            transition_table.push_str(&"Alphabet".center(buffer + 8));
            transition_table.push('|');
            transition_table.push_str(&"To".center(states_len.max(2) + buffer));
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
