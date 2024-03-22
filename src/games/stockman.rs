use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StockmanPos {
    pub node: usize,
}

pub struct StockmanHandler {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StockmanMove {
    LeftChild,
    RightChild,
}

impl GamePosition for StockmanPos {
    type Move = StockmanMove;

    fn startpos() -> Self {
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

impl GameHandler<StockmanPos, ()> for StockmanHandler {
    type Eval = i32;

    const EVAL_MINIMUM: i32 = -100;
    const EVAL_MAXIMUM: i32 = 100;
    const EVAL_EPSILON: i32 = 1;

    fn new(_: ()) -> Self {
        Self {}
    }

    fn get_legal_moves(
        &self,
        pos: StockmanPos,
    ) -> impl Iterator<Item = <StockmanPos as GamePosition>::Move> {
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

        if pos.node > 15 {
            LegalMoves::NoMoves
        } else {
            LegalMoves::HasMoves(
                std::iter::once(StockmanMove::LeftChild)
                    .chain(std::iter::once(StockmanMove::RightChild)),
            )
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
}
