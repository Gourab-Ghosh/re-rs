#![allow(unused_imports)]
#![allow(dead_code)]
// #![allow(unused)]

mod constants;
mod dfa;
mod nfa;
mod utils;

use constants::*;
use dfa::*;
use itertools::*;
use nfa::*;
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;
use std::ops::Range;
use utils::*;

fn test1() -> Result<(), CustomError> {
    let state1 = State::new('q', 0);
    let state2 = State::new('q', 1);
    let state3 = State::new('q', 2);
    let state4 = State::new('q', 3);

    // ab*a
    let mut dfa = DFA::new(
        HashSet::from([state1, state2, state3, state4]),
        HashSet::from_iter("ab".chars()),
        HashMap::from([
            ((state1, 'a'), state2),
            ((state1, 'b'), state4),
            ((state2, 'a'), state3),
            ((state2, 'b'), state2),
            ((state3, 'a'), state4),
            ((state3, 'b'), state4),
            ((state4, 'a'), state4),
            ((state4, 'b'), state4),
        ]),
        state1,
        HashSet::from([state3]),
    )?;
    println!("{dfa}");

    // let text = "bbabba";
    // let text = "abaaa";
    // let text = "aaaabaabaa";
    let text = "bbabbabbabbbaabaaaaba";
    // let text = "abaaaaba";

    for (slice_index, slice) in dfa.find_all(text)? {
        println!("{slice_index:?} -> {slice}");
    }

    // let slice_index = dfa.find(text);
    // let slice = slice_index.clone().map(|slice_index| &text[slice_index]);
    // println!("{slice_index:?} -> {slice:?}");

    Ok(())
}

fn test2() -> Result<(), CustomError> {
    let state1 = State::new('q', 0);
    let state2 = State::new('q', 1);
    let state3 = State::new('q', 2);
    let state4 = State::new('q', 3);
    let trapped_state = State::new('q', 4);

    // a|ab*a
    let mut dfa = DFA::new(
        HashSet::from([state1, state2, state3, state4, trapped_state]),
        HashSet::from_iter("ab".chars()),
        HashMap::from([
            ((state1, 'a'), state2),
            ((state1, 'b'), trapped_state),
            ((state2, 'a'), state3),
            ((state2, 'b'), state4),
            ((state3, 'a'), state3),
            ((state3, 'b'), state4),
            ((state4, 'a'), state3),
            ((state4, 'b'), state4),
            ((trapped_state, 'a'), trapped_state),
            ((trapped_state, 'b'), trapped_state),
        ]),
        state1,
        HashSet::from([state2, state3]),
    )?;
    println!("{dfa}");

    // let text = "aaaaa";
    // let text = "abaaa";
    // let text = "aaaabaabaa";
    // let text = "bbabbabbabbbaabaaaaba";
    let text = "bbabbabababbbbabbbababbababbbaabaaaaba";
    // let text = "aaba";

    for (slice_index, slice) in dfa.find_all(text)? {
        println!("{slice_index:?} -> {slice}");
    }

    // let slice_index = dfa.find(text);
    // let slice = slice_index.clone().map(|slice_index| &text[slice_index]);
    // println!("{slice_index:?} -> {slice:?}");

    Ok(())
}

fn test3() -> Result<(), CustomError> {
    let state1 = State::new('q', 0);
    let state2 = State::new('q', 1);
    let state3 = State::new('q', 2);
    let state4 = State::new('q', 3);
    let trapped_state = State::new('q', 4);

    // ab*a
    let dfa1 = DFA::new(
        HashSet::from([state1, state2, state3, state4]),
        HashSet::from_iter("ab".chars()),
        HashMap::from([
            ((state1, 'a'), state2),
            ((state1, 'b'), state4),
            ((state2, 'a'), state3),
            ((state2, 'b'), state2),
            ((state3, 'a'), state4),
            ((state3, 'b'), state4),
            ((state4, 'a'), state4),
            ((state4, 'b'), state4),
        ]),
        state1,
        HashSet::from([state3]),
    )?;

    // a|ab*a
    let dfa2 = DFA::new(
        HashSet::from([state1, state2, state3, state4, trapped_state]),
        HashSet::from_iter("ab".chars()),
        HashMap::from([
            ((state1, 'a'), state2),
            ((state1, 'b'), trapped_state),
            ((state2, 'a'), state3),
            ((state2, 'b'), state4),
            ((state3, 'a'), state3),
            ((state3, 'b'), state4),
            ((state4, 'a'), state3),
            ((state4, 'b'), state4),
            ((trapped_state, 'a'), trapped_state),
            ((trapped_state, 'b'), trapped_state),
        ]),
        state1,
        HashSet::from([state2, state3]),
    )?;

    let dfa_union = dfa1.union(&dfa2)?;

    println!("{}", dfa_union);

    let text = "bbabbabababbbbabbbababbababbbaabaaaaba";
    // let text = "aba";

    for (slice_index, slice) in dfa_union.clone().find_all(text)? {
        println!("{slice_index:?} -> {slice}");
    }

    // let dfa1_output: HashSet<(Range<usize>, &str)> = HashSet::from_iter(dfa1.find_all(text)?);
    // let dfa2_output: HashSet<(Range<usize>, &str)> = HashSet::from_iter(dfa2.find_all(text)?);
    // println!("{:?}", dfa1_output.intersection(&dfa2_output).collect_vec());
    // // println!("{}", dfa_union.clone().accepts(text)?);
    // println!("{:?}", dfa_union.clone().find_all(text)?.collect_vec());

    Ok(())
}

fn test4() -> Result<(), CustomError> {
    let state1 = State::new('q', 0);
    let state2 = State::new('q', 1);
    let state3 = State::new('q', 2);
    let state4 = State::new('q', 3);
    let trapped_state = State::new('q', 4);

    // ab*a
    let dfa1 = DFA::new(
        HashSet::from([state1, state2, state3, state4]),
        HashSet::from_iter("ab".chars()),
        HashMap::from([
            ((state1, 'a'), state2),
            ((state1, 'b'), state4),
            ((state2, 'a'), state3),
            ((state2, 'b'), state2),
            ((state3, 'a'), state4),
            ((state3, 'b'), state4),
            ((state4, 'a'), state4),
            ((state4, 'b'), state4),
        ]),
        state1,
        HashSet::from([state3]),
    )?;

    // a|ab*a
    let dfa2 = DFA::new(
        HashSet::from([state1, state2, state3, state4, trapped_state]),
        HashSet::from_iter("ab".chars()),
        HashMap::from([
            ((state1, 'a'), state2),
            ((state1, 'b'), trapped_state),
            ((state2, 'a'), state3),
            ((state2, 'b'), state4),
            ((state3, 'a'), state3),
            ((state3, 'b'), state4),
            ((state4, 'a'), state3),
            ((state4, 'b'), state4),
            ((trapped_state, 'a'), trapped_state),
            ((trapped_state, 'b'), trapped_state),
        ]),
        state1,
        HashSet::from([state2, state3]),
    )?;

    let dfa_intersection = dfa1.intersection(&dfa2)?;

    println!("{}", dfa_intersection);

    let text = "bbabbabababbbbabbbababbababbbaabaaaaba";
    // let text = "aba";

    for (slice_index, slice) in dfa_intersection.clone().find_all(text)? {
        println!("{slice_index:?} -> {slice}");
    }

    // let dfa1_output: HashSet<(Range<usize>, &str)> = HashSet::from_iter(dfa1.find_all(text)?);
    // let dfa2_output: HashSet<(Range<usize>, &str)> = HashSet::from_iter(dfa2.find_all(text)?);
    // println!("{:?}", dfa1_output.intersection(&dfa2_output).collect_vec());
    // // println!("{}", dfa_intersection.clone().accepts(text)?);
    // println!("{:?}", dfa_intersection.clone().find_all(text)?.collect_vec());

    Ok(())
}

fn test5() -> Result<(), CustomError> {
    let state0 = State::new('q', 0);
    let state1 = State::new('q', 1);
    let state2 = State::new('q', 2);
    let state3 = State::new('q', 3);
    let nfa = EpsilonNFA::new(
        HashSet::from([state0, state1, state2, state3]),
        HashSet::from([Some('a'), Some('b')]),
        HashMap::from([
            ((state0, Some('a')), HashSet::from([state1, state3])),
            ((state1, Some('a')), HashSet::from([state1, state2])),
            ((state1, Some('b')), HashSet::from([state1])),
        ]),
        HashSet::from([state0]),
        HashSet::from([state2, state3]),
    )?;

    let dfa = nfa.to_dfa();
    println!("{}", dfa);

    Ok(())
}

fn test6() -> Result<(), CustomError> {
    let state0 = State::new('q', 0);
    let state1 = State::new('q', 1);
    let state2 = State::new('q', 2);
    let state3 = State::new('q', 3);
    let mut nfa = EpsilonNFA::new(
        HashSet::from([state0, state1, state2, state3]),
        HashSet::from([Some('a'), Some('b')]),
        HashMap::from([
            ((state0, Some('a')), HashSet::from([state1, state3])),
            ((state1, Some('a')), HashSet::from([state1, state2])),
            ((state1, Some('b')), HashSet::from([state1])),
            ((state1, None), HashSet::from([state3])),
        ]),
        HashSet::from([state0, state1]),
        HashSet::from([state2, state3]),
    )?;

    println!("{nfa}");
    nfa.remove_epsilon_transitions();
    println!("{nfa}");

    let mut dfa = nfa.to_dfa();

    println!("{dfa}");
    dfa.minimize();
    println!("{dfa}");

    Ok(())
}

fn main() {
    test6().unwrap();
}
