use crate::prelude::*;

use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaChaRng;

use auto_enums::auto_enum;

// A representation of a node in a hypothetical game tree,
// which can have constant or non-constant fanout at each node.
// The only restrictions are that, if there is a known upper bound to fanout,
// then there will be no transpositions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HypTreePos {
    // The number of child nodes this node will spawn.
    pub fanout: usize,
    // The unique integer representing this node.
    pub node: usize,
}

impl GamePosition for HypTreePos {
    // A `Move` contains two pieces of information:
    // The number of places to the right to shift from the left-most child, and
    // The fanout of the child node created in this move.
    type Move = (usize, usize);
    type Params = usize;

    fn startpos(fanout: usize) -> Self {
        Self { fanout, node: 0 }
    }

    fn play_move(&self, mv: Self::Move) -> Self {
        let (fanout, shift) = mv;
        Self {
            fanout,
            node: self.node * self.fanout + shift,
        }
    }
}

#[derive(Debug)]
pub struct UnordIndHypTreeHandler {
    width: usize,
    // The `leaf_start` variable is an exclusive lower bound for leaf nodes.
    leaf_start: usize,
    node_values: Vec<i64>,
    leaf_count: u128,
}

pub struct HypTreeParams {
    // The maximum depth the hypothetical game tree will go to.
    pub depth: usize,
    // The maximum fanout of any node in the hypothetical game tree.
    pub width: usize,
    // The random seed to supply the handler to generate the random node values.
    pub seed: u64,
}

impl GameHandler<HypTreePos> for UnordIndHypTreeHandler {
    type Eval = i64;
    type Params = HypTreeParams;

    const EVAL_MINIMUM: i64 = -i64::MAX;
    const EVAL_MAXIMUM: i64 = i64::MAX;
    const EVAL_EPSILON: i64 = 1;

    fn new(params: HypTreeParams) -> Self {
        let HypTreeParams { depth, width, seed } = params;
        let mut rng = ChaChaRng::seed_from_u64(seed);
        // With a depth of `d` and a width/fanout of `w`, there are `w^d` leaf nodes.
        // All leaf nodes have a node value greater than or equal to the left-most leaf node.
        let mut leaf_start: usize = 0;
        for _ in 0..depth {
            leaf_start = leaf_start * width + 1;
        }
        leaf_start -= 1;
        let mut node_values: Vec<i64> = (1..=width.pow(depth as u32) as i64).collect();
        for i in (1..node_values.len()).rev() {
            let j = rng.gen_range(0..=i);
            (node_values[i], node_values[j]) = (node_values[j], node_values[i]);
        }
        Self {
            width,
            leaf_start,
            node_values,
            leaf_count: 0,
        }
    }

    #[auto_enum(Iterator)]
    fn get_legal_moves(
        &self,
        pos: HypTreePos,
    ) -> impl Iterator<Item = <HypTreePos as GamePosition>::Move> {
        if pos.node > self.leaf_start {
            std::iter::empty()
        } else {
            (1..=self.width).map(|shift| (self.width, shift))
        }
    }

    fn evaluate(&self, pos: HypTreePos, depth: usize, max_depth: usize) -> Self::Eval {
        if pos.node > self.leaf_start {
            let toggle = if ((max_depth - depth) & 1) == 0 {
                1
            } else {
                -1
            };
            self.node_values[pos.node - self.leaf_start - 1] * toggle
        } else {
            // `evaluate` should not be called on non-leaf nodes.
            0
        }
    }

    fn increment_leaf_count(&mut self) {
        self.leaf_count += 1;
    }

    fn get_leaf_count(&self) -> u128 {
        self.leaf_count
    }

    fn reset_leaf_count(&mut self) {
        self.leaf_count = 0;
    }
}
