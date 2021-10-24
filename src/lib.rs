mod graph;

extern crate streaming_iterator;
extern crate num;

#[macro_use]
extern crate enum_display_derive;

use streaming_iterator::StreamingIterator;
use std::fmt::Display;

pub trait Environment {
    type State: Sized;
    type Action: Sized;

    fn apply(&self, state: &mut Self::State, action: &Self::Action);
    fn rollback(&self, state: &mut Self::State, action: &Self::Action);
}

pub trait IsTerminalState<E: Environment> {
    fn is_terminal_state(&self, state: &E::State) -> bool;
}

pub trait WeightedState<E: Environment> {
    type Weight: num::Float;

    fn calc_weight(&mut self, state: &mut E::State) -> Self::Weight;
}

pub trait ActionsGenerator<E: Environment> {
    type ActionsIterator: Iterator<Item=E::Action> + Sized;

    fn generate_actions(&mut self, state: &E::State) -> Self::ActionsIterator;
}


#[derive(Display, PartialEq)]
pub enum LastVisitAction {
    New,
    EnterNew,
    EnterOld,
    Done
}

pub struct RecursiveStateVisitor<'a, E: Environment, A: ActionsGenerator<E>> {
    stack: Vec<(A::ActionsIterator, E::Action)>,
    state: E::State,
    env: &'a E,
    agent: A,
    last_action_in: bool,
    need_expand_current: bool,
    is_finished: bool
}

impl<'a, E: Environment, A: ActionsGenerator<E>> RecursiveStateVisitor<'a, E, A> {
    pub fn new(initial: E::State, env: &'a E, agent: A) -> RecursiveStateVisitor<'a, E, A> {
        RecursiveStateVisitor { stack: vec![], state: initial, env, agent, last_action_in: true, need_expand_current: true, is_finished: false}
    }

    #[inline(always)]
    pub fn stop_expand_current(&mut self) {
        self.need_expand_current = false;
    }
}

impl<'a, E: Environment, A: ActionsGenerator<E>> StreamingIterator for RecursiveStateVisitor<'a, E, A> {
    type Item = E::State;

    #[inline(always)]
    fn advance(&mut self) {
        if !self.need_expand_current {
            self.env.rollback(&mut self.state, &self.stack.last().unwrap().1);
            self.last_action_in = false;
            self.need_expand_current = true;
        } else if self.last_action_in {
            let mut actions = self.agent.generate_actions(&self.state);
            if let Some(action) = actions.next() {
                self.env.apply(&mut self.state, &action);
                self.stack.push((actions, action));
            }
        } else {
            if let Some(iter) = self.stack.last_mut() {
                if let Some(action) = iter.0.next() {
                    self.env.apply(&mut self.state, &action);
                    self.last_action_in = true;
                    iter.1 = action;
                } else {
                    self.env.rollback(&mut self.state, &iter.1);
                    self.stack.pop();
                }
            } else {
                self.is_finished = true;
            }
        }
    }

    #[inline(always)]
    fn get(&self) -> Option<&Self::Item> {
        if self.is_finished {
            None
        } else {
            Some(&self.state)
        }
    }
}

pub trait ShouldContinueSearch<E: Environment> {
    fn should_continue(&mut self, state: &E::State) -> bool;
}

pub struct RecursiveStateGenerator<'a, E: Environment + IsTerminalState<E>, A: ActionsGenerator<E> + ShouldContinueSearch<E>> {
    visitor: RecursiveStateVisitor<'a, E, A>,
    env: &'a E
}

impl<'a, E: Environment + IsTerminalState<E>, A: ActionsGenerator<E> + ShouldContinueSearch<E>> RecursiveStateGenerator<'a, E, A> {
    pub fn new(initial: E::State, env: &'a E, agent: A) -> Self {
        RecursiveStateGenerator { visitor: RecursiveStateVisitor::new(initial, env, agent), env }
    }
}

impl<'a, E: Environment + IsTerminalState<E>, A: ActionsGenerator<E> + ShouldContinueSearch<E>> StreamingIterator for RecursiveStateGenerator<'a, E, A> {
    type Item = E::State;

    #[inline(always)]
    fn advance(&mut self) {
        self.visitor.advance();
        while !self.visitor.is_finished {
            if self.visitor.last_action_in && self.env.is_terminal_state(&self.visitor.state) {
                self.visitor.stop_expand_current();
                break;
            } else if !self.visitor.agent.should_continue(&self.visitor.state) {
                self.visitor.stop_expand_current();
            }
            self.visitor.advance()
        }
    }

    #[inline(always)]
    fn get(&self) -> Option<&Self::Item> {
        self.visitor.get()
    }
}
