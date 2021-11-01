extern crate ai_agents;
extern crate rand;

use rand::{Rng, thread_rng};
use ai_agents::{Environment, RecursiveStateGenerator, ActionsGenerator, WeightedState};
use rand::rngs::ThreadRng;
use std::iter::{once, Once, FromIterator};
use ai_agents::simulated_annealing::simulated_annealing::AnnealingSolver;

#[derive(Clone)]
struct Path {
    p: Vec<usize>,
    cur_w: i32
}

impl Path {
    fn new(len: usize) -> Self {
        Path{p: (0..len).collect(), cur_w: 0}
    }

    fn swap(&mut self, n1: usize, n2: usize) {
        self.p.swap(n1, n2);
    }

}

#[derive(Clone)]
struct PairOfNodes {
    n1: usize,
    n2: usize
}

struct GraphEnvironment {
    weights: Vec<Vec<i32>>
}

impl Environment for GraphEnvironment {
    type State = Path;
    type Action = PairOfNodes;

    fn apply(&self, state: &mut Self::State, action: &Self::Action) {
        let (n1, n2) = if action.n1 > action.n2 { (action.n1, action.n2) } else { (action.n2, action.n1) };
        let l11 = state.p[n1+1];
        let l12 = state.p[n1-1];
        let l21 = state.p[n2+1];
        let l22 = state.p[n2-1];
        let l1 = state.p[n1];
        let l2 = state.p[n2];
        state.swap(n1, n2);
        let mut diff = 0;
        if n1 - n2 == 1 {
            diff = self.weights[l11][l1] + self.weights[l22][l2]
                - self.weights[l11][l2] - self.weights[l22][l1];
        } else {
            diff = self.weights[l11][l1] + self.weights[l12][l1] + self.weights[l21][l2] + self.weights[l22][l2]
                - self.weights[l21][l1] - self.weights[l22][l1] - self.weights[l11][l2] - self.weights[l12][l2];
        }
        state.cur_w += diff;
    }

    fn rollback(&self, state: &mut Self::State, action: &Self::Action) {
        // basically the same as the apply
        self.apply(state, action);
    }
}
struct GraphAgent<'a> {
    env: &'a GraphEnvironment,
    generator: ThreadRng
}

impl<'a> GraphAgent<'a> {
    fn new(env: &'a GraphEnvironment) -> Self {
        Self{env, generator: thread_rng()}
    }

    fn calc_path_weight(&self, p: &Path) -> i32 {
        let mut res = 0;
        for i in 0..p.p.len()-1 {
           res = res + self.env.weights[p.p[i]][p.p[i+1]];
        }
        res
    }
}

impl<'a> WeightedState<GraphEnvironment> for GraphAgent<'a> {
    fn calc_weight(&mut self, state: &Path) -> f64 {
        state.cur_w as f64
    }
}

impl<'a> ActionsGenerator<GraphEnvironment> for GraphAgent<'a> {
    type ActionsIterator = Once<PairOfNodes>;

    fn generate_actions(&mut self, state: &Path) -> Self::ActionsIterator {
        let len = self.env.weights.len() - 2;
        once(PairOfNodes{n1: self.generator.gen::<usize>() % len + 1, n2: self.generator.gen::<usize>() % len + 1})
    }
}


#[test]
fn test_env_and_agents() {
    let weights = vec![
        vec![0,0,1,0,0],
        vec![0,0,0,1,1],
        vec![1,0,0,1,0],
        vec![0,1,1,0,0],
        vec![0,1,0,0,0]];
    let env = GraphEnvironment{weights};
    let mut p = Path::new(5);
    let mut agent = GraphAgent::new(&env);
    p.cur_w = -agent.calc_path_weight(&p);
    env.apply(&mut p, &PairOfNodes{n1: 1, n2: 3});
    assert_eq!(p.p[3], 1);
    assert_eq!(p.p[1], 3);
    assert_eq!(p.cur_w, -2);
    println!("{} {} {} {}", env.weights[p.p[0]][p.p[1]], env.weights[p.p[1]][p.p[2]], env.weights[p.p[2]][p.p[3]],env.weights[p.p[3]][p.p[4]]);
    env.apply(&mut p, &PairOfNodes{n1: 1, n2: 2});
    assert_eq!(p.p[1], 2);
    assert_eq!(p.p[2], 3);
    assert_eq!(p.p[3], 1);
    println!("{} {} {} {}", env.weights[p.p[0]][p.p[1]], env.weights[p.p[1]][p.p[2]], env.weights[p.p[2]][p.p[3]],env.weights[p.p[3]][p.p[4]]);
    assert_eq!(p.cur_w, -4);
}

#[test]
fn test_simulated_annealing() {
    let weights = vec![
        vec![0,0,1,0,0],
        vec![0,0,0,1,1],
        vec![1,0,0,1,0],
        vec![0,1,1,0,0],
        vec![0,1,0,0,0]];
    let env = GraphEnvironment{weights};
    let mut p = Path::new(5);
    let mut agent = GraphAgent::new(&env);
    p.cur_w = -agent.calc_path_weight(&p);
    let mut solver = AnnealingSolver::new(p, 100.0);
    let avg = solver.estimate_fluctuation(&env, &mut agent, 20);
    println!("estimated avg = {}", avg);
    assert!(avg < 10.0);
    solver.run(&env, &mut agent, &mut |x| { x - 0.5 }, 1.0);
    assert_eq!(agent.calc_weight(solver.get_best_state()), -4.0);
}
