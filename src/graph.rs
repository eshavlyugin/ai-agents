mod graph {
    use ::{Environment, ActionsGenerator};
    use ShouldContinueSearch;
    use streaming_iterator::{convert, StreamingIterator};

    pub trait Graph {
        type Node: Sized;
        type Edge: Sized;
        type NodeIterator: Iterator<Item=Self::Node>;
        type EdgeIterator: Iterator<Item=Self::Edge>;

        fn nodes(&self) -> Self::NodeIterator;
        fn edges(&self, node: &Self::Node) -> Self::EdgeIterator;
        fn edge_begin(e: &Self::Edge) -> Self::Node;
        fn edge_end(e: &Self::Edge) -> Self::Node;
    }

    struct GraphEnvironment<G: Graph> {
        graph: G
    }

    impl<G: Graph> Environment for GraphEnvironment<G> {
        type State = G::Node;
        type Action = G::Edge;

        fn apply(&self, state: &mut G::Node, action: &G::Edge) {
            *state = G::edge_begin(action);
        }

        fn rollback(&self, state: &mut G::Node, action: &G::Edge) {
            *state = G::edge_end(action);
        }
    }

    struct DfsAgent<'a, G: Graph> {
        graph: &'a mut G
    }

    impl<'a, G: Graph> ActionsGenerator<GraphEnvironment<G>> for DfsAgent<'a, G> {
        type ActionsIterator = G::EdgeIterator;

        fn generate_actions(&mut self, state: &G::Node) -> G::EdgeIterator {
            self.graph.edges(state)
        }
    }

    impl<'a, G: Graph> ShouldContinueSearch<GraphEnvironment<G>> for DfsAgent<'a, G> {
        fn should_continue(&mut self, state: &G::Node) -> bool {
            false
        }
    }

    pub fn is_tree<G: Graph>(g: G) -> bool {
        false
    }
}