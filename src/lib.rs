extern crate streaming_iterator;
extern crate num;

#[macro_use]
extern crate enum_display_derive;

use streaming_iterator::StreamingIterator;
use std::iter::Peekable;
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


#[derive(Display)]
pub enum LastVisitAction {
    New,
    EnterNew,
    EnterOld,
    Done
}

pub struct RecursiveStateVisitor<'a, E: Environment, A: ActionsGenerator<E>> {
    stack: Vec<Peekable<A::ActionsIterator>>,
    state: E::State,
    env: &'a E,
    agent: &'a mut A,
    last_action: LastVisitAction,
    need_expand_current: bool
}

impl<'a, E: Environment, A: ActionsGenerator<E>> RecursiveStateVisitor<'a, E, A> {
    pub fn new(initial: E::State, env: &'a E, agent: &'a mut A) -> RecursiveStateVisitor<'a, E, A> {
        RecursiveStateVisitor {stack: vec![], state: initial, env, agent, last_action: LastVisitAction::New, need_expand_current: true}
    }

    fn stop_expand_current(&mut self) {
        self.need_expand_current = false;
    }

    fn apply_action(env: &E, last_action: &mut LastVisitAction, is_action_performed: &mut bool, state: &mut E::State, action: &E::Action) {
        env.apply(state, action);
        *last_action = LastVisitAction::EnterNew;
        *is_action_performed = true;
    }

    fn rollback_action(env: &E, stack: &mut Vec<Peekable<A::ActionsIterator>>, last_action: &mut LastVisitAction, is_action_performed: &mut bool, state: &mut E::State) {
        if let Some(prev_iter) = stack.last_mut() {
            env.rollback(state, &prev_iter.next().unwrap());
            *last_action = LastVisitAction::EnterOld;
            *is_action_performed = true;
        }
    }
}

impl<'a, E: Environment, A: ActionsGenerator<E>> StreamingIterator for RecursiveStateVisitor<'a, E, A> {
    type Item = E::State;

    fn advance(&mut self) {
        match &self.last_action {
            LastVisitAction::New => {
                self.last_action = LastVisitAction::EnterNew;
            },
            LastVisitAction::EnterNew|LastVisitAction::EnterOld => {
                let mut is_action_performed = false;
                let is_enter = if let LastVisitAction::EnterNew = self.last_action { true } else { false };
                if is_enter {
                    if self.need_expand_current {
                        let mut actions = self.agent.generate_actions(&self.state).peekable();
                        if let Some(action) = actions.peek() {
                            Self::apply_action(&self.env, &mut self.last_action, &mut is_action_performed, &mut self.state, action);
                            self.stack.push(actions);
                        }
                    } else {
                        Self::rollback_action(&self.env, &mut self.stack, &mut self.last_action, &mut is_action_performed, &mut self.state);
                    }
                }
                if !is_action_performed {
                    if let Some(iter) = self.stack.last_mut() {
                        if let Some(action) = iter.peek() {
                            Self::apply_action(&self.env, &mut self.last_action, &mut is_action_performed, &mut self.state, action);
                        } else {
                            self.stack.pop();
                            Self::rollback_action(&self.env, &mut self.stack, &mut self.last_action, &mut is_action_performed, &mut self.state);
                        }
                    }
                }
                if !is_action_performed {
                    self.last_action = LastVisitAction::Done;
                }
                self.need_expand_current = true;
            },
            LastVisitAction::Done => {}
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        match &self.last_action {
            LastVisitAction::Done => None,
            _ => Some(&self.state)
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
    pub fn new(initial: E::State, env: &'a E, agent: &'a mut A) -> Self {
        RecursiveStateGenerator { visitor: RecursiveStateVisitor::new(initial, env, agent), env }
    }
}

impl<'a, E: Environment + IsTerminalState<E>, A: ActionsGenerator<E> + ShouldContinueSearch<E>> StreamingIterator for RecursiveStateGenerator<'a, E, A> {
    type Item = E::State;

    fn advance(&mut self) {
        self.visitor.advance();
        while let LastVisitAction::EnterNew|LastVisitAction::EnterOld = self.visitor.last_action {
            if self.env.is_terminal_state(&self.visitor.state) {
                self.visitor.stop_expand_current();
                break;
            } else if !self.visitor.agent.should_continue(&self.visitor.state) {
                self.visitor.stop_expand_current();
            }
            self.visitor.advance()
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        match &self.visitor.last_action {
            LastVisitAction::Done => None,
            _ => Some(&self.visitor.state)
        }
    }
}
