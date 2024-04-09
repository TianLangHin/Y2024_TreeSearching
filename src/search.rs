use crate::prelude::*;

use rayon::prelude::*;

use std::collections::BinaryHeap;
use std::time::Instant;

// Return type of all searching algorithms,
// consisting of the calculated heuristic evaluation of the position
// and the series of moves that the evaluation corresponds to.
pub type EvalAndPV<THandler, TPosition, const SIZE: usize> = (
    <THandler as GameHandler<TPosition>>::Eval,
    [Option<<TPosition as GamePosition>::Move>; SIZE],
);

// To enable the counting of leaf node evaluation,
// we implement all searching algorithms as member functions
// of a `Searcher` object, which separates the need for counting
// away from the `GameHandler` and `GamePosition` traits,
// and also satisfying the borrow checker.
// This construct is required as legal move generation returns an `Iterator`,
// which would require an immutable borrow of the `GameHandler`.
pub struct Searcher {
    leaf_count: u128,
}

// Suggestion from #[warn(clippy::new_without_default)]
impl Default for Searcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Searcher {
    // The only internal state of `Searcher` that gets mutated incrementally
    // as an algorithm runs is the number of leaf nodes evaluated to this point.
    pub fn new() -> Self {
        Self { leaf_count: 0 }
    }

    // Functions for the algorithms to increment the `leaf_count`
    // and for the user to reset this count in between separate algorithm calls.
    pub fn increment_leaf_count(&mut self) {
        self.leaf_count += 1;
    }

    pub fn get_leaf_count(&self) -> u128 {
        self.leaf_count
    }

    pub fn reset_leaf_count(&mut self) {
        self.leaf_count = 0;
    }

    // Utility functions for testing legal move generation and calculating
    // the total number of leaf nodes in a maximal tree of a given depth.
    // The terminology of `perft` is borrowed from the functionality of chess engines
    // that carries out this functionality, commonly used for legal move generation debugging.
    // Since many game trees are very large in size, we give parallel implementations as well,
    // with the side effect that verbose parallel options do not have a move printing order guarantee.
    fn perft<THandler, TPosition>(depth: usize, pos: TPosition, handler: &THandler) -> u128
    where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        if depth == 1 {
            handler.get_legal_moves(pos).count() as u128
        } else {
            handler
                .get_legal_moves(pos)
                .map(|mv| Self::perft(depth - 1, pos.play_move(mv), handler))
                .sum()
        }
    }

    pub fn perft_div_serial<THandler, TPosition>(
        depth: usize,
        pos: TPosition,
        handler: &THandler,
        verbose: bool,
    ) where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        if verbose {
            println!("Serial perft (Depth = {})", depth);
        }
        if depth == 1 {
            let s = Instant::now();
            println!("Nodes searched: {}", handler.get_legal_moves(pos).count());
            println!("Time elapsed: {} ms", s.elapsed().as_millis());
            return;
        }
        let s = Instant::now();
        let sum: u128 = if verbose {
            handler
                .get_legal_moves(pos)
                .map(|mv| {
                    let num = Self::perft(depth - 1, pos.play_move(mv), handler);
                    println!("{:?}: {num}", mv);
                    num
                })
                .sum()
        } else {
            handler
                .get_legal_moves(pos)
                .map(|mv| Self::perft(depth - 1, pos.play_move(mv), handler))
                .sum()
        };
        println!("Nodes searched: {sum}");
        println!("Time elapsed {} ms", s.elapsed().as_millis());
    }

    // std::marker::Sync is not enforced in the prelude traits,
    // but is required for the parallel perft implementation.
    pub fn perft_div_parallel<THandler, TPosition>(
        depth: usize,
        pos: TPosition,
        handler: &THandler,
        verbose: bool,
    ) where
        THandler: GameHandler<TPosition> + Sync,
        TPosition: GamePosition + Sync,
        <TPosition as GamePosition>::Move: Sync,
    {
        if verbose {
            println!("Serial perft (Depth = {})", depth);
        }
        if depth == 1 {
            let s = Instant::now();
            println!("Nodes searched: {}", handler.get_legal_moves(pos).count());
            println!("Time elapsed: {} ms", s.elapsed().as_millis());
            return;
        }
        let s = Instant::now();
        let sum: u128 = if verbose {
            handler
                .get_legal_moves(pos)
                .collect::<Vec<_>>()
                .par_iter()
                .map(|&mv| {
                    let num = Self::perft(depth - 1, pos.play_move(mv), handler);
                    println!("{:?}: {num}", mv);
                    num
                })
                .sum()
        } else {
            handler
                .get_legal_moves(pos)
                .collect::<Vec<_>>()
                .par_iter()
                .map(|&mv| Self::perft(depth - 1, pos.play_move(mv), handler))
                .sum()
        };
        println!("Nodes searched: {sum}");
        println!("Time elapsed {} ms", s.elapsed().as_millis());
    }

    // Replication of algorithms described in Muszycka & Shinghal (1985).

    // Algorithm A.
    pub fn branch_and_bound<THandler, TPosition, const MAX_DEPTH: usize>(
        &mut self,
        handler: &THandler,
        pos: TPosition,
        depth: usize,
        bound: <THandler as GameHandler<TPosition>>::Eval,
    ) -> EvalAndPV<THandler, TPosition, MAX_DEPTH>
    where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        // A node `MAX_DEPTH` plies ahead of the root is considered a leaf.
        // Statement 5.
        if depth == 0 {
            self.increment_leaf_count();
            return (handler.evaluate(pos, depth, MAX_DEPTH), [None; MAX_DEPTH]);
        }

        // Statement 4.
        let mut move_iter = handler.get_legal_moves(pos);

        if let Some(mut mv) = move_iter.next() {
            // Statement 6.
            let mut m = <THandler as GameHandler<TPosition>>::EVAL_MINIMUM;
            let mut pv = [None; MAX_DEPTH];

            loop {
                // Statement 9.
                let (t, mut line) = self.branch_and_bound::<THandler, TPosition, MAX_DEPTH>(
                    handler,
                    pos.play_move(mv),
                    depth - 1,
                    -m,
                );
                let t = -t;
                line[MAX_DEPTH - depth] = Some(mv);

                if t > m {
                    m = t;
                    pv = line;
                }

                // Statement 10.
                if m >= bound {
                    return (m, line);
                }

                if let Some(new_mv) = move_iter.next() {
                    mv = new_mv;
                } else {
                    break;
                }
            }

            (m, pv)
        } else {
            // Statement 5.
            self.increment_leaf_count();
            (handler.evaluate(pos, depth, MAX_DEPTH), [None; MAX_DEPTH])
        }
    }

    // Algorithm B.
    pub fn alpha_beta<THandler, TPosition, const MAX_DEPTH: usize>(
        &mut self,
        handler: &THandler,
        pos: TPosition,
        depth: usize,
        alpha: <THandler as GameHandler<TPosition>>::Eval,
        beta: <THandler as GameHandler<TPosition>>::Eval,
    ) -> EvalAndPV<THandler, TPosition, MAX_DEPTH>
    where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        // A node `MAX_DEPTH` plies ahead of the root is considered a leaf.
        // Statement 5.
        if depth == 0 {
            self.increment_leaf_count();
            return (handler.evaluate(pos, depth, MAX_DEPTH), [None; MAX_DEPTH]);
        }

        // Statement 4.
        let mut move_iter = handler.get_legal_moves(pos);

        if let Some(mut mv) = move_iter.next() {
            // Statement 6.
            let mut m = alpha;
            let mut pv = [None; MAX_DEPTH];

            loop {
                // Statement 9.
                let (t, mut line) = self.alpha_beta::<THandler, TPosition, MAX_DEPTH>(
                    handler,
                    pos.play_move(mv),
                    depth - 1,
                    -beta,
                    -m,
                );
                let t = -t;
                line[MAX_DEPTH - depth] = Some(mv);

                if t > m {
                    m = t;
                    pv = line;
                }

                // Statement 10.
                if m >= beta {
                    return (m, line);
                }

                if let Some(new_mv) = move_iter.next() {
                    mv = new_mv;
                } else {
                    break;
                }
            }

            (m, pv)
        } else {
            // Statement 5.
            self.increment_leaf_count();
            (handler.evaluate(pos, depth, MAX_DEPTH), [None; MAX_DEPTH])
        }
    }

    // Algorithm C.
    pub fn p_alpha_beta<THandler, TPosition, const MAX_DEPTH: usize>(
        &mut self,
        handler: &THandler,
        pos: TPosition,
        depth: usize,
    ) -> EvalAndPV<THandler, TPosition, MAX_DEPTH>
    where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        // A node `MAX_DEPTH` plies ahead of the root is considered a leaf.
        // Statement 5.
        if depth == 0 {
            self.increment_leaf_count();
            return (handler.evaluate(pos, depth, MAX_DEPTH), [None; MAX_DEPTH]);
        }

        // Statement 4.
        let mut move_iter = handler.get_legal_moves(pos);

        if let Some(mv) = move_iter.next() {
            // Statement 6.
            let (mut m, mut pv) = self.p_alpha_beta::<THandler, TPosition, MAX_DEPTH>(
                handler,
                pos.play_move(mv),
                depth - 1,
            );
            m = -m;
            pv[MAX_DEPTH - depth] = Some(mv);

            // Statement 7.
            for mv in move_iter {
                let next_pos = pos.play_move(mv);

                // Statement 9.
                let t = -self
                    .f_alpha_beta::<THandler, TPosition, MAX_DEPTH>(
                        handler,
                        next_pos,
                        depth - 1,
                        -m - <THandler as GameHandler<TPosition>>::EVAL_EPSILON,
                        -m,
                    )
                    .0;

                // Statement 10.
                if t > m {
                    // Statement 11.
                    // In Muszycka & Shinghal (1985), this statement was erroneously written as
                    // `m = -alphabeta(p_i, -MAXINT, -t);` as opposed to
                    // `m = -falphabeta(p_i, -MAXINT, -t);`. Fishburn & Finkel (1980)
                    // originally describe this algorithm correctly.
                    let (t, mut line) = self.f_alpha_beta::<THandler, TPosition, MAX_DEPTH>(
                        handler,
                        next_pos,
                        depth - 1,
                        <THandler as GameHandler<TPosition>>::EVAL_MINIMUM,
                        -t,
                    );
                    m = -t;
                    line[MAX_DEPTH - depth] = Some(mv);
                    pv = line;
                }
            }

            (m, pv)
        } else {
            // Statement 5.
            self.increment_leaf_count();
            (handler.evaluate(pos, depth, MAX_DEPTH), [None; MAX_DEPTH])
        }
    }

    pub fn f_alpha_beta<THandler, TPosition, const MAX_DEPTH: usize>(
        &mut self,
        handler: &THandler,
        pos: TPosition,
        depth: usize,
        alpha: <THandler as GameHandler<TPosition>>::Eval,
        beta: <THandler as GameHandler<TPosition>>::Eval,
    ) -> EvalAndPV<THandler, TPosition, MAX_DEPTH>
    where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        // A node `MAX_DEPTH` plies ahead of the root is considered a leaf.
        // Statement 5.
        if depth == 0 {
            self.increment_leaf_count();
            return (handler.evaluate(pos, depth, MAX_DEPTH), [None; MAX_DEPTH]);
        }

        // Statement 4.
        let mut move_iter = handler.get_legal_moves(pos);

        if let Some(mut mv) = move_iter.next() {
            // Statement 6.
            let mut m = <THandler as GameHandler<TPosition>>::EVAL_MINIMUM;
            let mut pv = [None; MAX_DEPTH];

            loop {
                // Statement 9.
                let (t, mut line) = self.f_alpha_beta::<THandler, TPosition, MAX_DEPTH>(
                    handler,
                    pos.play_move(mv),
                    depth - 1,
                    -beta,
                    -std::cmp::max(m, alpha),
                );
                let t = -t;
                line[MAX_DEPTH - depth] = Some(mv);

                if t > m {
                    m = t;
                    pv = line;
                }

                // Statement 10.
                if m >= beta {
                    return (m, line);
                }

                if let Some(new_mv) = move_iter.next() {
                    mv = new_mv;
                } else {
                    break;
                }
            }

            (m, pv)
        } else {
            // Statement 5.
            self.increment_leaf_count();
            (handler.evaluate(pos, depth, MAX_DEPTH), [None; MAX_DEPTH])
        }
    }

    // Algorithm D.
    pub fn pvs<THandler, TPosition, const MAX_DEPTH: usize>(
        &mut self,
        handler: &THandler,
        pos: TPosition,
        depth: usize,
        alpha: <THandler as GameHandler<TPosition>>::Eval,
        beta: <THandler as GameHandler<TPosition>>::Eval,
    ) -> EvalAndPV<THandler, TPosition, MAX_DEPTH>
    where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        // A node `MAX_DEPTH` plies ahead of the root is considered a leaf.
        // Statement 5.
        if depth == 0 {
            self.increment_leaf_count();
            return (handler.evaluate(pos, depth, MAX_DEPTH), [None; MAX_DEPTH]);
        }

        // Statement 4.
        let mut move_iter = handler.get_legal_moves(pos);

        if let Some(mv) = move_iter.next() {
            // Statement 6.
            let (mut m, mut pv) = self.pvs::<THandler, TPosition, MAX_DEPTH>(
                handler,
                pos.play_move(mv),
                depth - 1,
                -beta,
                -alpha,
            );
            m = -m;
            pv[MAX_DEPTH - depth] = Some(mv);

            // Statement 7.
            if m < beta {
                // Statement 8.
                for mv in move_iter {
                    // Statement 10.
                    let bound = std::cmp::max(m, alpha);

                    let next_pos = pos.play_move(mv);

                    // Statement 11.
                    let t = -self
                        .pvs::<THandler, TPosition, MAX_DEPTH>(
                            handler,
                            next_pos,
                            depth - 1,
                            -bound - <THandler as GameHandler<TPosition>>::EVAL_EPSILON,
                            -bound,
                        )
                        .0;

                    // Statement 12.
                    if t > m {
                        // Statement 13.
                        let (value, mut line) = self.pvs::<THandler, TPosition, MAX_DEPTH>(
                            handler,
                            next_pos,
                            depth - 1,
                            -beta,
                            -t,
                        );
                        m = -value;
                        line[MAX_DEPTH - depth] = Some(mv);
                        pv = line;
                    }
                    // Statement 14.
                    if m >= beta {
                        return (m, pv);
                    }
                }
            }

            (m, pv)
        } else {
            // Statement 5.
            self.increment_leaf_count();
            (handler.evaluate(pos, depth, MAX_DEPTH), [None; MAX_DEPTH])
        }
    }

    // Algorithm E.
    pub fn scout<THandler, TPosition, const MAX_DEPTH: usize>(
        &mut self,
        handler: &THandler,
        pos: TPosition,
        depth: usize,
    ) -> EvalAndPV<THandler, TPosition, MAX_DEPTH>
    where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        // A node `MAX_DEPTH` plies ahead of the root is considered a leaf.
        // Statement 5.
        if depth == 0 {
            self.increment_leaf_count();
            return (handler.evaluate(pos, depth, MAX_DEPTH), [None; MAX_DEPTH]);
        }

        // Statement 4.
        let mut move_iter = handler.get_legal_moves(pos);

        if let Some(mv) = move_iter.next() {
            // Statement 6.
            let (mut m, mut pv) = self.scout::<THandler, TPosition, MAX_DEPTH>(
                handler,
                pos.play_move(mv),
                depth - 1,
            );
            m = -m;
            pv[MAX_DEPTH - depth] = Some(mv);

            // Statement 7.
            let op = true;

            // Statement 8.
            for mv in move_iter {
                let next_pos = pos.play_move(mv);

                // Statement 9.
                if !self.test::<THandler, TPosition>(
                    handler,
                    next_pos,
                    depth - 1,
                    MAX_DEPTH,
                    -m,
                    !op,
                ) {
                    let (new_m, mut line) = self.scout::<THandler, TPosition, MAX_DEPTH>(
                        handler,
                        next_pos,
                        depth - 1,
                    );
                    let new_m = -new_m;
                    line[MAX_DEPTH - depth] = Some(mv);
                    m = new_m;
                    pv = line;
                }
            }

            (m, pv)
        } else {
            // Statement 5.
            self.increment_leaf_count();
            (handler.evaluate(pos, depth, MAX_DEPTH), [None; MAX_DEPTH])
        }
    }

    pub fn test<THandler, TPosition>(
        &mut self,
        handler: &THandler,
        pos: TPosition,
        depth: usize,
        max_depth: usize,
        v: <THandler as GameHandler<TPosition>>::Eval,
        op: bool,
    ) -> bool
    where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        // A node `max_depth` plies ahead of the root is considered a leaf.
        // Statement 5.
        if depth == 0 {
            // Statements 6-9.
            self.increment_leaf_count();
            return if op {
                handler.evaluate(pos, depth, max_depth) >= v
            } else {
                handler.evaluate(pos, depth, max_depth) > v
            };
        }

        // Statement 4.
        let mut move_iter = handler.get_legal_moves(pos);

        if let Some(mut mv) = move_iter.next() {
            loop {
                // Statement 11.
                if !self.test::<THandler, TPosition>(
                    handler,
                    pos.play_move(mv),
                    depth - 1,
                    max_depth,
                    -v,
                    !op,
                ) {
                    return true;
                }

                if let Some(new_mv) = move_iter.next() {
                    mv = new_mv;
                } else {
                    break;
                }
            }
            // Statement 13.
            false
        } else {
            // Statements 6-9.
            self.increment_leaf_count();
            if op {
                handler.evaluate(pos, depth, max_depth) >= v
            } else {
                handler.evaluate(pos, depth, max_depth) > v
            }
        }
    }

    // Algorithm F.
    pub fn sss<THandler, TPosition, const MAX_DEPTH: usize>(
        &mut self,
        handler: &THandler,
        root: TPosition,
        depth: usize,
    ) -> EvalAndPV<THandler, TPosition, MAX_DEPTH>
    where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        // The `State` data structure for use in the SSS* algorithm is defined here,
        // since this function is only called once at the root due to its iterative nature.
        // It is not defined earlier as it is only used by this algorithm and will not be returned either.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        enum State<TPos, TEval, TMove, const SIZE: usize>
        where
            TPos: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
            TEval: Clone + Copy + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord,
            TMove: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
        {
            Live {
                node: TPos,
                merit: (TEval, [Option<TMove>; SIZE]),
                depth: usize,
                line: [Option<TMove>; SIZE],
                iteration: usize,
            },
            Solved {
                node: TPos,
                merit: (TEval, [Option<TMove>; SIZE]),
                depth: usize,
                line: [Option<TMove>; SIZE],
                iteration: usize,
            },
        }

        impl<TPos, TEval, TMove, const SIZE: usize> State<TPos, TEval, TMove, SIZE>
        where
            TPos: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
            TEval: Clone + Copy + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord,
            TMove: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
        {
            fn merit(&self) -> (TEval, [Option<TMove>; SIZE]) {
                match *self {
                    Self::Solved {
                        node: _,
                        merit,
                        depth: _,
                        line: _,
                        iteration: _,
                    } => merit,
                    Self::Live {
                        node: _,
                        merit,
                        depth: _,
                        line: _,
                        iteration: _,
                    } => merit,
                }
            }

            fn depth(&self) -> usize {
                match *self {
                    Self::Solved {
                        node: _,
                        merit: _,
                        depth,
                        line: _,
                        iteration: _,
                    } => depth,
                    Self::Live {
                        node: _,
                        merit: _,
                        depth,
                        line: _,
                        iteration: _,
                    } => depth,
                }
            }

            fn line(&self) -> [Option<TMove>; SIZE] {
                match *self {
                    Self::Solved {
                        node: _,
                        merit: _,
                        depth: _,
                        line,
                        iteration: _,
                    } => line,
                    Self::Live {
                        node: _,
                        merit: _,
                        depth: _,
                        line,
                        iteration: _,
                    } => line,
                }
            }

            fn iteration(&self) -> usize {
                match *self {
                    Self::Solved {
                        node: _,
                        merit: _,
                        depth: _,
                        line: _,
                        iteration,
                    } => iteration,
                    Self::Live {
                        node: _,
                        merit: _,
                        depth: _,
                        line: _,
                        iteration,
                    } => iteration,
                }
            }

            fn is_max_player(&self, max_depth: usize) -> bool {
                ((max_depth - self.depth()) & 1) == 0
            }
        }

        impl<TPos, TEval, TMove, const SIZE: usize> PartialOrd for State<TPos, TEval, TMove, SIZE>
        where
            TPos: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
            TEval: Clone + Copy + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord,
            TMove: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
        {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl<TPos, TEval, TMove, const SIZE: usize> Ord for State<TPos, TEval, TMove, SIZE>
        where
            TPos: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
            TEval: Clone + Copy + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord,
            TMove: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
        {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.merit()
                    .0
                    .cmp(&other.merit().0)
                    .then_with(|| self.iteration().cmp(&other.iteration()))
            }
        }

        let mut open: BinaryHeap<
            State<
                TPosition,
                <THandler as GameHandler<TPosition>>::Eval,
                <TPosition as GamePosition>::Move,
                MAX_DEPTH,
            >,
        > = BinaryHeap::new();

        open.push(State::Live {
            node: root,
            merit: (
                <THandler as GameHandler<TPosition>>::EVAL_MAXIMUM,
                [None; MAX_DEPTH],
            ),
            depth,
            line: [None; MAX_DEPTH],
            iteration: 0,
        });

        let mut i: usize = 1;

        while let Some(state) = open.pop() {
            match state {
                State::Solved {
                    node: n,
                    merit: (h, pv),
                    depth: d,
                    line: mut l,
                    iteration: _,
                } => {
                    if d == MAX_DEPTH {
                        return (h, pv);
                    }
                    let mut parent = root;
                    let path_length = MAX_DEPTH - d - 1;
                    for mv in l.iter().take(path_length) {
                        parent = parent.play_move(mv.unwrap());
                    }
                    if state.is_max_player(MAX_DEPTH) {
                        if let Some(next_move) = handler
                            .get_legal_moves(parent)
                            .skip_while(|&mv| parent.play_move(mv) != n)
                            .nth(1)
                        {
                            l[path_length] = Some(next_move);
                            for i in path_length + 1..MAX_DEPTH {
                                l[i] = None;
                            }
                            // Case 2.
                            open.push(State::Live {
                                node: parent.play_move(next_move),
                                merit: (h, pv),
                                depth: d,
                                line: l,
                                iteration: i,
                            });
                        } else {
                            // Case 3.
                            open.push(State::Solved {
                                node: parent,
                                merit: (h, pv),
                                depth: d + 1,
                                line: l,
                                iteration: i,
                            });
                        }
                    } else {
                        // Case 1.
                        open.retain(|&state| {
                            state
                                .line()
                                .iter()
                                .zip(l.iter())
                                .take(path_length)
                                .any(|(&best, &discard)| best != discard)
                        });
                        open.push(State::Solved {
                            node: parent,
                            merit: (h, pv),
                            depth: d + 1,
                            line: l,
                            iteration: i,
                        });
                    }
                }
                State::Live {
                    node: n,
                    merit: (h, pv),
                    depth: d,
                    line: l,
                    iteration: _,
                } => {
                    let mut legal_moves = handler.get_legal_moves(n);
                    if d == 0 {
                        // To account for the negamax construct in conjunction with SSS* node evaluation.
                        self.increment_leaf_count();
                        let eval = if ((MAX_DEPTH - depth) & 1) == 0 {
                            handler.evaluate(n, depth, MAX_DEPTH)
                        } else {
                            -handler.evaluate(n, depth, MAX_DEPTH)
                        };
                        // Extension of Case 4. `MAX_DEPTH` plies from root is considered leaf.
                        open.push(State::Solved {
                            node: n,
                            merit: if h < eval { (h, pv) } else { (eval, l) },
                            depth: d,
                            line: l,
                            iteration: i,
                        });
                    } else if let Some(first_move) = legal_moves.next() {
                        let mut line = l;
                        line[MAX_DEPTH - d] = Some(first_move);
                        if state.is_max_player(MAX_DEPTH) {
                            // Case 6.
                            open.push(State::Live {
                                node: n.play_move(first_move),
                                merit: (h, pv),
                                depth: d - 1,
                                line,
                                iteration: i,
                            });
                            for mv in legal_moves {
                                line[MAX_DEPTH - d] = Some(mv);
                                open.push(State::Live {
                                    node: n.play_move(mv),
                                    merit: (h, pv),
                                    depth: d - 1,
                                    line,
                                    iteration: i,
                                });
                            }
                        } else {
                            // Case 5.
                            open.push(State::Live {
                                node: n.play_move(first_move),
                                merit: (h, pv),
                                depth: d - 1,
                                line,
                                iteration: i,
                            });
                        }
                    } else {
                        // To account for the negamax construct in conjunction with SSS* node evaluation.
                        self.increment_leaf_count();
                        let eval = if ((MAX_DEPTH - depth) & 1) == 0 {
                            handler.evaluate(n, depth, MAX_DEPTH)
                        } else {
                            -handler.evaluate(n, depth, MAX_DEPTH)
                        };
                        // Next legal move is `None` on first attempt: leaf node. Thus, Case 4.
                        open.push(State::Solved {
                            node: n,
                            merit: if h < eval { (h, pv) } else { (eval, l) },
                            depth: d,
                            line: l,
                            iteration: i,
                        });
                    }
                }
            }
            i += 1;
        }
        panic!("State space operator is faulty");
    }
}
