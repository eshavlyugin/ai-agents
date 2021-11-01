use std::hash::Hash;
use streaming_iterator::StreamingIterator;
use std::collections::HashMap;

struct LayeredPuzzleSolver<S: Hash + Eq + Clone, V: Default> {
    current_layer: HashMap<S, V>
}

impl<S: Hash + Eq + Clone, V: Default> LayeredPuzzleSolver<S, V> {
    fn new<F: Fn(&S, &mut V), I: StreamingIterator<Item=S>>(initializer: F, states: I) -> Self {
        let mut layer = HashMap::new();
        states.for_each(|s| initializer(s, layer.entry(s.clone()).or_default()));
        LayeredPuzzleSolver{ current_layer : layer }
    }

    /*fn process_layer<'a, T: Fn(&V, &mut V), I: 'a + StreamingIterator<Item=S>, G: Fn(&'a S) -> I>(self, transform: T, generator: G) -> Self {
    let mut new_layer = HashMap::new();
    for (s, v) in self.current_layer.into_iter() {
    generator(&s).for_each(|new_s| transform(&v, new_layer.entry(new_s.clone()).or_default()));
    }
    Self { current_layer: new_layer }
    }*/
}

