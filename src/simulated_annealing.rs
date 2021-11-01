pub mod simulated_annealing {

    extern crate rand;

    use ::{Environment, ActionsGenerator};
    use WeightedState;
    use self::rand::{thread_rng, Rng};

    pub struct AnnealingSolver<E: Environment> {
        current_state: E::State,
        best_state: E::State,
        temp: f64
    }

    impl<E: Environment> AnnealingSolver<E> {
        pub fn new(state: E::State, init_temp: f64) -> Self {
            Self{current_state: state.clone(), best_state: state.clone(), temp: init_temp}
        }

        pub fn step<A: ActionsGenerator<E>  + WeightedState<E>>(&mut self, env: &E, agent: &mut A, accept_always: bool) {
            let mut actions = agent.generate_actions(&self.current_state);
            let mut rng = thread_rng();
            if let Some(action) = actions.next() {
                let prev_w = agent.calc_weight(&self.current_state);
                env.apply(&mut self.current_state, &action);
                let new_w = agent.calc_weight(&self.current_state);
                let rand_val = rng.gen::<f64>();
                if accept_always || rand_val >= ((new_w - prev_w) / &self.temp).exp() {
                    env.rollback(&mut self.current_state, &action);
                }
            }
        }

        pub fn run<F: FnMut(f64) -> f64, A: ActionsGenerator<E>  + WeightedState<E>>(&mut self, env: &E, agent: &mut A, cooldown: &mut F, final_temp: f64) {
            while self.temp > final_temp {
                self.step(env, agent, false);
                self.temp = cooldown(self.temp);
                if agent.calc_weight(&self.current_state) < agent.calc_weight(&self.best_state) {
                    self.best_state = self.current_state.clone();
                }
            }
        }

        pub fn estimate_fluctuation<A: ActionsGenerator<E>  + WeightedState<E>>(&mut self, env: &E, agent: &mut A, steps: usize) -> f64 {
            let mut avg = 0.0;
            let prev_w = agent.calc_weight(&mut self.current_state);
            for _ in 0..steps {
                self.step(env, agent, true);
                avg += (prev_w - agent.calc_weight(&mut self.current_state)).abs();
            }
            avg / (steps as f64)
        }

        pub fn get_best_state(&self) -> &E::State {
            &self.best_state
        }
    }

}
