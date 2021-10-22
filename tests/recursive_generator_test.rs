extern crate streaming_iterator;
extern crate num;
extern crate ai_agents;

use ai_agents::{Environment, RecursiveStateGenerator, ActionsGenerator, IsTerminalState, ShouldContinueSearch};
use streaming_iterator::StreamingIterator;

struct ZeroOneEnvironment {
    max_size: usize
}

struct ZeroOneAgent {
}


impl<'a> ShouldContinueSearch<ZeroOneEnvironment> for ZeroOneAgent {
    fn should_continue(&mut self, _state: &Vec<bool>) -> bool {
        true
    }
}

impl Environment for ZeroOneEnvironment {
    type State = Vec<bool>;
    type Action = bool;

    fn apply(&self, state: &mut Self::State, action: &Self::Action) {
        state.push(*action);
    }
    fn rollback(&self, state: &mut Self::State, _: &Self::Action) { state.pop(); }
}

impl IsTerminalState<ZeroOneEnvironment> for ZeroOneEnvironment {
    fn is_terminal_state(&self, state: &Vec<bool>) -> bool {
        state.len() >= self.max_size
    }
}

impl ActionsGenerator<ZeroOneEnvironment> for ZeroOneAgent {
    type ActionsIterator = std::vec::IntoIter<bool>;

    fn generate_actions(&mut self, _: &Vec<bool>) -> Self::ActionsIterator {
        return vec![true, false].into_iter();
    }
}

#[test]
fn test_zero_one_recursive_generator() {
    let env = ZeroOneEnvironment{max_size: 3};
    let mut gen = RecursiveStateGenerator::new(vec![], &env, ZeroOneAgent{});
    let mut count = 0;
    while let Some(_) = gen.next() {
        count = count + 1;
    }

    assert_eq!(count, 8);
}

#[test]
fn test_zero_one_initial_is_terminal() {
    let env = ZeroOneEnvironment{max_size: 0};
    let mut gen = RecursiveStateGenerator::new(vec![], &env, ZeroOneAgent{});

    let mut count = 0;
    while let Some(_) = gen.next() {
        count = count + 1;
    }

    assert_eq!(count, 1);
}

#[test]
fn test_zero_one_recursive_generator_take_while() {
    let env = ZeroOneEnvironment{max_size: 3};
    let mut gen = RecursiveStateGenerator::new(vec![], &env, ZeroOneAgent{}).take_while(|s| s[0]);

    let mut count = 0;
    while let Some(_) = gen.next() {
        count = count + 1;
    }

    assert_eq!(count, 4);
}