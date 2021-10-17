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

pub trait VisitState<E: Environment> {
    fn on_enter_state(&mut self, state: &E::State) -> bool;
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
pub enum VisitorState {
    New,
    Enter,
    Leave,
    Done
}

pub struct RecursiveStateVisitor<'a, E: Environment, A: VisitState<E> + ActionsGenerator<E>> {
    stack: Vec<Peekable<A::ActionsIterator>>,
    state: E::State,
    env: &'a E,
    agent: &'a mut A,
    pub last_state: VisitorState
}

impl<'a, E: Environment, A: VisitState<E> + ActionsGenerator<E>> RecursiveStateVisitor<'a, E, A> {
    pub fn new(initial: E::State, env: &'a E, agent: &'a mut A) -> RecursiveStateVisitor<'a, E, A> {
        RecursiveStateVisitor {stack: vec![], state: initial, env, agent, last_state: VisitorState::New}
    }
}

impl<'a, E: Environment, A: VisitState<E> + ActionsGenerator<E>> StreamingIterator for RecursiveStateVisitor<'a, E, A> {
    type Item = E::State;

    fn advance(&mut self) {
        match &self.last_state {
            VisitorState::New => {
                self.last_state = VisitorState::Enter;
            },
            VisitorState::Enter => {
                if self.agent.on_enter_state(&self.state) {
                    self.stack.push(self.agent.generate_actions(&self.state).peekable());
                    if let Some(action) = self.stack.last_mut().unwrap().peek() {
                        self.env.apply(&mut self.state, &action);
                        self.last_state = VisitorState::Enter;
                    } else {
                        self.stack.pop();
                        self.last_state = VisitorState::Leave;
                    }
                } else {
                    self.last_state = VisitorState::Leave
                }
            },
            VisitorState::Leave => {
                if let Some(iter) = self.stack.last_mut() {
                    self.env.rollback(&mut self.state, &iter.next().unwrap());
                    if let Some(action) = iter.peek() {
                        self.env.apply(&mut self.state, &action);
                        self.last_state = VisitorState::Enter
                    } else {
                        self.stack.pop();
                        self.last_state = VisitorState::Leave
                    }
                } else {
                    self.last_state = VisitorState::Done;
                }
            },
            VisitorState::Done => {}
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        match &self.last_state {
            VisitorState::Done => None,
            _ => Some(&self.state)
        }
    }
}

pub struct RecursiveStateGenerator<'a, E: Environment + IsTerminalState<E>, A: VisitState<E> + ActionsGenerator<E>> {
    visitor: RecursiveStateVisitor<'a, E, A>,
    env: &'a E
}

impl<'a, E: Environment + IsTerminalState<E>, A: VisitState<E> + ActionsGenerator<E>> RecursiveStateGenerator<'a, E, A> {
    pub fn new(initial: E::State, env: &'a E, agent: &'a mut A) -> Self {
        RecursiveStateGenerator { visitor: RecursiveStateVisitor::new(initial, env, agent), env }
    }

    fn stop_advance(&self) -> bool {
        match self.visitor.last_state {
            VisitorState::Enter => self.env.is_terminal_state(&self.visitor.state),
            VisitorState::Done => true,
            _ => false
        }
    }
}

impl<'a, E: Environment + IsTerminalState<E>, A: VisitState<E> + ActionsGenerator<E>> StreamingIterator for RecursiveStateGenerator<'a, E, A> {
    type Item = E::State;

    fn advance(&mut self) {
        self.visitor.advance();
        while !self.stop_advance()
        {
            self.visitor.advance();
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        match &self.visitor.last_state {
            VisitorState::Done => None,
            _ => Some(&self.visitor.state)
        }
    }
}
