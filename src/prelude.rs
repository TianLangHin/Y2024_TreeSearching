pub trait GamePosition: Clone + Copy + std::fmt::Debug + PartialEq + Eq {
    type Move: Clone + Copy + std::fmt::Debug + PartialEq + Eq;
    type Params;

    fn startpos(params: Self::Params) -> Self;
    fn play_move(&self, mv: Self::Move) -> Self;
}

pub trait GameHandler<TPosition>
where
    TPosition: GamePosition,
{
    type Eval: Clone
        + Copy
        + std::fmt::Debug
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::ops::Add<Output = Self::Eval>
        + std::ops::Sub<Output = Self::Eval>
        + std::ops::Neg<Output = Self::Eval>;
    type Params;

    // Must be ensured that EVAL_MINIMUM == -EVAL_MAXIMUM.
    const EVAL_MINIMUM: Self::Eval;
    const EVAL_MAXIMUM: Self::Eval;
    const EVAL_EPSILON: Self::Eval;

    fn new(params: Self::Params) -> Self;
    fn get_legal_moves(
        &self,
        pos: TPosition,
    ) -> impl Iterator<Item = <TPosition as GamePosition>::Move>;
    fn evaluate(&self, pos: TPosition, depth: usize, max_depth: usize) -> Self::Eval;
}
