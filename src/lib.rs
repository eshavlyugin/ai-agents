mod graph;
pub mod simulated_annealing;
mod dynamic_programming;
mod alpha_beta;

extern crate streaming_iterator;
extern crate num;

#[macro_use]
extern crate enum_display_derive;

use streaming_iterator::StreamingIterator;
use std::fmt::Display;

pub trait Environment {
    type State: Sized + Clone;
    type Action: Sized + Clone;

    fn apply(&self, state: &mut Self::State, action: &Self::Action);
    fn rollback(&self, state: &mut Self::State, action: &Self::Action);
}

pub trait IsTerminalState<E: Environment> {
    fn is_terminal_state(&self, state: &E::State) -> bool;
}

pub trait WeightedState<E: Environment> {
    fn calc_weight(&mut self, state: &E::State) -> f64;
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
    last_action: LastVisitAction,
    last_applied_action: Option<E::Action>,
    need_expand_current: bool,
}

impl<'a, E: Environment, A: ActionsGenerator<E>> RecursiveStateVisitor<'a, E, A> {
    pub fn new(initial: E::State, env: &'a E, agent: A) -> RecursiveStateVisitor<'a, E, A> {
        RecursiveStateVisitor { stack: vec![], state: initial, env, agent, last_action: LastVisitAction::EnterNew, last_applied_action: None, need_expand_current: true}
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
            let (_, action) = self.stack.last().unwrap();
            self.env.rollback(&mut self.state, &action);
            self.last_action = LastVisitAction::EnterOld;

            self.need_expand_current = true;
        } else if self.last_action == LastVisitAction::EnterNew {
            let mut actions = self.agent.generate_actions(&self.state);
            if let Some(action) = actions.next() {
                self.env.apply(&mut self.state, &action);
                self.stack.push((actions, action));
            }
        } else {
            if let Some(iter) = self.stack.last_mut() {
                if let Some(action) = iter.0.next() {
                    self.env.apply(&mut self.state, &action);
                    self.last_action = LastVisitAction::EnterNew;
                    iter.1 = action;
                } else {
                    self.env.rollback(&mut self.state, &iter.1);
                    self.stack.pop();
                    if self.stack.is_empty() {
                        self.last_action = LastVisitAction::Done;
                    } else {
                        self.last_action = LastVisitAction::EnterOld;
                    }
                }
            } else {
            }
        }
    }

    #[inline(always)]
    fn get(&self) -> Option<&Self::Item> {
        if self.last_action == LastVisitAction::Done {
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
        while self.visitor.last_action != LastVisitAction::Done {
            if self.visitor.last_action == LastVisitAction::EnterNew && self.env.is_terminal_state(&self.visitor.state) {
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
