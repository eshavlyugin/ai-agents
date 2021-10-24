extern crate streaming_iterator;
extern crate num;
extern crate ai_agents;
extern crate alloc;
extern crate core;

use ai_agents::{Environment, RecursiveStateGenerator, ActionsGenerator, IsTerminalState, ShouldContinueSearch};
use streaming_iterator::StreamingIterator;
use std::ops::Range;

struct SeqEnvironment {
    max_size: usize
}

struct SeqAgent {
}


impl<'a> ShouldContinueSearch<SeqEnvironment> for SeqAgent {
    fn should_continue(&mut self, _state: &Vec<u8>) -> bool {
        true
    }
}

impl Environment for SeqEnvironment {
    type State = Vec<u8>;
    type Action = u8;

    fn apply(&self, state: &mut Self::State, action: &Self::Action) {
        state.push(*action);
    }
    fn rollback(&self, state: &mut Self::State, _: &Self::Action) { state.pop(); }
}

impl IsTerminalState<SeqEnvironment> for SeqEnvironment {
    fn is_terminal_state(&self, state: &Vec<u8>) -> bool {
        state.len() >= self.max_size
    }
}

impl ActionsGenerator<SeqEnvironment> for SeqAgent {
    type ActionsIterator = Range<u8>;

    fn generate_actions(&mut self, _: &Vec<u8>) -> Self::ActionsIterator {
        (1u8..9u8).into_iter()
    }
}

#[test]
fn test_zero_one_recursive_generator() {
    let env = SeqEnvironment{max_size: 10};
    let gen = RecursiveStateGenerator::new(vec![], &env, SeqAgent{});
    let mut count: usize = 0;
    gen.for_each(|_s| count = count + 1 );

    assert_eq!(count, 1073741824);
}

#[test]
#[ignore]
fn test_zero_one_initial_is_terminal() {
    let env = SeqEnvironment{max_size: 0};
    let gen = RecursiveStateGenerator::new(vec![], &env, SeqAgent{});

    let mut count = 0;
    gen.for_each(|_s| count = count + 1 );

    assert_eq!(count, 1);
}

fn generate_01_rec<F: FnMut()>(state: &mut Vec<u8>, env: &SeqEnvironment, agent: &mut SeqAgent, func: &mut F) {
    if env.is_terminal_state(state) {
        func();
        return;
    } else {
        agent.generate_actions(state).for_each(|action| {
            env.apply(state, &action);
            generate_01_rec(state, env, agent, func);
            env.rollback(state, &action);
        });
    }
}

#[test]
fn test_zero_one_simple() {
    let env = SeqEnvironment{max_size: 10};
    let mut count: usize = 0;
    generate_01_rec(&mut vec![], &env, &mut SeqAgent{}, &mut || { count += 1 } );
    assert_eq!(count, 1073741824);
}

#[test]
fn test_zero_one_recursive_generator_take_while() {
    let env = SeqEnvironment{max_size: 3};
    //let mut gen = RecursiveStateGenerator::new(vec![], &env, ZeroOneAgent{}).take_while(|s| s[0]);
    let gen = RecursiveStateGenerator::new(vec![], &env, SeqAgent{}).take_while(|s| s[0] != 4);

    let mut count = 0;
    gen.for_each(|_s| count = count + 1 );

    assert_eq!(count, 192);
}