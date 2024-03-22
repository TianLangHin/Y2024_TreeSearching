use crate::prelude::*;

use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaChaRng;

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Uniform2bWidePos {
    pub node: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uniform2bWideMove {
    LeftChild,
    RightChild,
}

impl GamePosition for Uniform2bWidePos {
    type Move = Uniform2bWideMove;

    fn startpos() -> Self {
        Self { node: 1 }
    }

    fn play_move(&self, mv: Self::Move) -> Self {
        match mv {
            Uniform2bWideMove::LeftChild => Self {
                node: self.node << 1,
            },
            Uniform2bWideMove::RightChild => Self {
                node: (self.node << 1) + 1,
            },
        }
    }
}

pub struct Uniform2bWideHandler {
    leaf_start: u32,
    pub node_values: BTreeMap<u32, i32>,
}

pub struct Uniform2bWideParams {
    pub depth: u32,
    pub seed: u64,
}

impl GameHandler<Uniform2bWidePos, Uniform2bWideParams> for Uniform2bWideHandler {
    type Eval = i32;

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
                ((rng.next_u32() & 0xffff) as i32)
                    * if (rng.next_u32() & 1) == 1 { -1 } else { 1 }
            );
        }
        Self { leaf_start: 1 << depth, node_values }
    }

    fn get_legal_moves(
        &self,
        pos: Uniform2bWidePos,
    ) -> impl Iterator<Item = <Uniform2bWidePos as GamePosition>::Move> {
        enum LegalMoves<T> {
            NoMoves,
            HasMoves(T),
        }

        impl<T> Iterator for LegalMoves<T>
        where
            T: Iterator,
        {
            type Item = <T as Iterator>::Item;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Self::NoMoves => None,
                    Self::HasMoves(x) => x.next(),
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                match self {
                    Self::NoMoves => (0, Some(0)),
                    Self::HasMoves(x) => x.size_hint(),
                }
            }
        }

        if pos.node >= self.leaf_start {
            LegalMoves::NoMoves
        } else {
            LegalMoves::HasMoves(
                std::iter::once(Uniform2bWideMove::LeftChild)
                    .chain(std::iter::once(Uniform2bWideMove::RightChild)),
            )
        }
    }

    fn evaluate(&self, pos: Uniform2bWidePos, _depth: usize, _max_depth: usize) -> Self::Eval {
        match self.node_values.get(&pos.node) {
            Some(&n) => n,
            None => i32::MAX,
        }
    }
}