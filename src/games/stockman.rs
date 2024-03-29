use crate::prelude::*;

use auto_enums::auto_enum;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StockmanPos {
    pub node: usize,
}

pub struct StockmanHandler {
    leaf_count: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StockmanMove {
    LeftChild,
    RightChild,
}

impl GamePosition for StockmanPos {
    type Move = StockmanMove;
    type Params = ();

    fn startpos(_: ()) -> Self {
        Self { node: 1 }
    }

    fn play_move(&self, mv: Self::Move) -> Self {
        match mv {
            StockmanMove::LeftChild => Self {
                node: self.node << 1,
            },
            StockmanMove::RightChild => Self {
                node: (self.node << 1) + 1,
            },
        }
    }
}

impl GameHandler<StockmanPos> for StockmanHandler {
    type Eval = i32;
    type Params = ();

    const EVAL_MINIMUM: i32 = -100;
    const EVAL_MAXIMUM: i32 = 100;
    const EVAL_EPSILON: i32 = 1;

    fn new(_: Self::Params) -> Self {
        Self { leaf_count: 0 }
    }

    #[auto_enum(Iterator)]
    fn get_legal_moves(
        &self,
        pos: StockmanPos,
    ) -> impl Iterator<Item = <StockmanPos as GamePosition>::Move> {
        if pos.node > 15 {
            std::iter::empty()
        } else {
            std::iter::once(StockmanMove::LeftChild)
                .chain(std::iter::once(StockmanMove::RightChild))
        }
    }

    fn evaluate(&self, pos: StockmanPos, _depth: usize, _max_depth: usize) -> Self::Eval {
        match pos.node {
            16 => 30,
            17 => 54,
            18 => 21,
            19 => 73,
            20 => 9,
            21 => 71,
            22 => 43,
            23 => 91,
            24 => 28,
            25 => 94,
            26 => 78,
            27 => 52,
            28 => 22,
            29 => 35,
            30 => 53,
            31 => 80,
            _ => i32::MAX,
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
