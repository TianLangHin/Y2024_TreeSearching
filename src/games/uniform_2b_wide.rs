use crate::prelude::*;

use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaChaRng;

use auto_enums::auto_enum;

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Uniform2bWidePos {
    pub node: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uniform2bWideMove {
    Left,
    Right,
}

impl GamePosition for Uniform2bWidePos {
    type Move = Uniform2bWideMove;
    type Params = ();

    fn startpos(_: ()) -> Self {
        Self { node: 1 }
    }

    fn play_move(&self, mv: Self::Move) -> Self {
        match mv {
            Uniform2bWideMove::Left => Self {
                node: self.node << 1,
            },
            Uniform2bWideMove::Right => Self {
                node: (self.node << 1) + 1,
            },
        }
    }
}

pub struct Uniform2bWideHandler {
    leaf_start: u32,
    node_values: BTreeMap<u32, i32>,
}

pub struct Uniform2bWideParams {
    pub depth: u32,
    pub seed: u64,
}

// So that `node_values` does not have to be a public field
impl Uniform2bWideHandler {
    fn get_node_values(&self) -> BTreeMap<u32, i32> {
        self.node_values.clone()
    }
}

impl GameHandler<Uniform2bWidePos> for Uniform2bWideHandler {
    type Eval = i32;
    type Params = Uniform2bWideParams;

    const EVAL_MINIMUM: i32 = -i32::MAX;
    const EVAL_MAXIMUM: i32 = i32::MAX;
    const EVAL_EPSILON: i32 = 1;

    fn new(params: Uniform2bWideParams) -> Self {
        let Uniform2bWideParams { depth, seed } = params;
        let mut node_values: BTreeMap<u32, i32> = BTreeMap::new();
        let mut rng: ChaChaRng = ChaChaRng::seed_from_u64(seed);
        for node in 1 << depth..1 << (depth + 1) {
            node_values.insert(
                node,
                rng.gen_range(-100..=100),
            );
        }
        Self {
            leaf_start: 1 << depth,
            node_values,
        }
    }

    #[auto_enum(Iterator)]
    fn get_legal_moves(
        &self,
        pos: Uniform2bWidePos,
    ) -> impl Iterator<Item = <Uniform2bWidePos as GamePosition>::Move> {
        if pos.node >= self.leaf_start {
            std::iter::empty()
        } else {
            std::iter::once(Uniform2bWideMove::Left)
                .chain(std::iter::once(Uniform2bWideMove::Right))
        }
    }

    fn evaluate(&self, pos: Uniform2bWidePos, _depth: usize, _max_depth: usize) -> Self::Eval {
        match self.node_values.get(&pos.node) {
            Some(&n) => n,
            None => i32::MAX,
        }
    }
}
