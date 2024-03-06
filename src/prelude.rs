pub trait GamePosition: Clone + Copy + std::fmt::Debug + PartialEq + Eq {
    type Move: Clone + Copy + std::fmt::Debug + PartialEq + Eq;
    fn startpos() -> Self;
    fn play_move(&self, _mv: Self::Move) -> Self;
}

pub trait GameHandler<GP>
where
    GP: GamePosition,
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
    const EVAL_MINIMUM: Self::Eval;
    const EVAL_MAXIMUM: Self::Eval;
    const EVAL_EPSILON: Self::Eval;
    fn new() -> Self;
    fn get_legal_moves(&self, _pos: GP) -> impl Iterator<Item = <GP as GamePosition>::Move>;
    fn evaluate(&self, _pos: GP) -> Self::Eval;
}
