// These trait definitions provide a framework for different game tree searching algorithms
// to operate on different game tree topologies without having to tailor algorithms to game trees individually.
// This framework accommodates only for two-player zero-sum turn-based perfect-information games.


// The `GamePosition` trait is by implemented by the data structure
// that represents a node in the intended game tree (i.e. it represents a single game state).
// In some alternative implementations of game states, a static value may be modified in-place
// using `play_move` and `undo_move` functions, eliminating the need for copying game states.

// However, to accommodate for searching algorithms that are iterative in nature,
// this trait definition forces `Copy` to be implemented.
// Additionally, `Eq` is to enable functionalities such as verification of algorithm correctness
// and for the utility of checking whether a position has already been searched, among others.
pub trait GamePosition: Clone + Copy + std::fmt::Debug + PartialEq + Eq {

    // The transition from one game state to another is represented by a `Move`.
    // `Copy` is enforced to allow for game lines to be efficiently generated and recorded.
    // `Eq` is enforced to allow for the functionality of checking whether to game paths are identical.
    type Move: Clone + Copy + std::fmt::Debug + PartialEq + Eq;

    // When initialising a game state, some parameters may be needed as input.
    // The `Params` associated type allows information of a structure specific
    // to the `GamePosition` implementation to be fed into `startpos` for initialisation.
    // If no information needs to be passed, `Params` can be the unit type `()`.
    type Params;

    // The associated function to construct and initialise a game state,
    // with information taken from the passed `Params` instance.
    fn startpos(params: Self::Params) -> Self;

    // To ensure searching algorithms perform as efficiently as possible,
    // the operation of generating a new game state from a previous game state and a given move
    // should be somewhat incremental in nature, and lightweight enough that it can be
    // represented as a member function belonging to the game state data structure itself,
    // rather than needing a reference to the `GameHandler`.
    // Instead of mutating the game state in-place, this function generates a new game state object.
    fn play_move(&self, mv: Self::Move) -> Self;
}


// The `GameHandler` trait is implemented by an object, which should not be copied or moved.
// In game tree searching functions, the functionalities it provides should be accessed
// through an immutable reference. Hence, neither `Clone` nor `Copy` is needed.
// The object that implements `GameHandler` handles computationally intensive or complicated
// operations that are essential for searching the game tree, such as legal move generation.

// The use of a generic type parameter `TPosition` as opposed to an annotated type
// allows for one object to implement `GameHandler` for different kinds of game states,
// which can be used, for example, to represent different variants of the same game
// that have similar or identical rules for generating legal moves and evaluating positions.
// A `GameHandler` should only operate on valid game states,
// hence `TPosition` must implement `GamePosition`.
pub trait GameHandler<TPosition>
where
    TPosition: GamePosition,
{
    // The heuristic evaluation of a game state is not necessarily inherent to the game tree itself,
    // but rather is a conceptual construct built on top an existing game tree, meaningful only
    // within the scope of a game strategy. Thus, `Eval` is defined as an associated type
    // in the `GameHandler` trait rather than the `GamePosition` trait.

    // `Copy` is enforced so evaluation results can be passed to and from function calls efficiently.
    // `Eq` and `Ord` are needed to enable comparison between two heuristic evaluation results,
    // which allows algorithms to make decisions based on these values.
    // `Add` and `Sub` is enforced to allow these values to be adjusted manually within algorithms,
    // for usage such as window adjustment.
    // `Neg` is enforced to adopt the negamax framework, relying on the
    // two-player zero-sum turn-based perfect-information properties of this game tree.
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

    // To initialise the game handler object, some input parameters may be needed.
    // The `Params` associated type allows information of a structure specific
    // to the `GameHandler` implementation to be fed into `new` for initialisation.
    // If no information needs to be passed, `Params` can be the unit type `()`.
    type Params;


    // The `GameHandler` implementation must also define constants that are of the same type
    // as the associated type `Eval`, which will be used to represent certain concepts or game states.

    // The following behaviour must be ensured, as it is a logical error otherwise:
    // For the player to move, if a position is irrecoverably lost, `evaluate` returns `EVAL_MINIMUM`.
    // For the player to move, if a position is irrecoverably won, `evaluate` returns `EVAL_MAXIMUM`.
    // `EVAL_MINIMUM == -EVAL_MAXIMUM`.
    const EVAL_MINIMUM: Self::Eval;
    const EVAL_MAXIMUM: Self::Eval;

    // This constant is often used as the size of an aspiration window, and should be a relatively small value.
    // It is traditionally used to represent the smallest unit that the heuristic value can change by.
    const EVAL_EPSILON: Self::Eval;


    // The associated function to construct and initialise the `GameHandler` object,
    // with information taken from the passed `Params` instance.
    fn new(params: Self::Params) -> Self;

    // The below function definition requires Rust 1.75.0 or above.
    // It returns an Iterator object that generates a series of `Move` objects
    // instead of a collection of objects such as `Vec`, to avoid expensive collection operations
    // being built into searching algorithms themselves.

    // Generates all legal moves from a given game state.
    // Since legal move generation may require some amount of precomputation and storage of values
    // outside each game state representation,
    // `GameHandler` handles legal move generation rather than `GamePosition`.
    fn get_legal_moves(
        &self,
        pos: TPosition,
    ) -> impl Iterator<Item = <TPosition as GamePosition>::Move>;

    // This function returns the static heuristic evaluation function for a given game state,
    // from the perspective of the player to move in the given position.
    // The parameter `max_depth` is the maximum number of plies currently being searched ahead in the game tree.
    // The parameter `depth` is the number of plies away from depth termination the current position is.
    // These values are required to be given to the `evaluate` function to allow for frameworks where
    // a quicker path to victory can be numerically represented as more favourable than a longer path to victory.
    fn evaluate(&self, pos: TPosition, depth: usize, max_depth: usize) -> Self::Eval;
}
