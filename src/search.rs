use crate::prelude::*;
use std::collections::BinaryHeap;

// Return type of all searching algorithms,
// consisting of the calculated heuristic evaluation of the position
// and the series of moves that the evaluation corresponds to.
pub type MoveAndPV<THandler, TPosition, const SIZE: usize> = (
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
    pub fn new() -> Self {
        Self { leaf_count: 0 }
    }

    pub fn increment_leaf_count(&mut self) {
        self.leaf_count += 1;
    }

    pub fn get_leaf_count(&self) -> u128 {
        self.leaf_count
    }

    pub fn reset_leaf_count(&mut self) {
        self.leaf_count = 0;
    }

    // Replication of algorithms described in Muszycka & Shinghal (1985).

    // Algorithm A.
    pub fn branch_and_bound<THandler, TPosition, const SIZE: usize>(
        &mut self,
        handler: &THandler,
        pos: TPosition,
        depth: usize,
        max_depth: usize,
        bound: <THandler as GameHandler<TPosition>>::Eval,
    ) -> MoveAndPV<THandler, TPosition, SIZE>
    where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        // A node `max_depth` plies ahead of the root is considered a leaf.
        // Statement 5.
        if depth == 0 {
            self.increment_leaf_count();
            return (handler.evaluate(pos, depth, max_depth), [None; SIZE]);
        }

        // Statement 4.
        let mut move_iter = handler.get_legal_moves(pos);

        if let Some(mut mv) = move_iter.next() {
            // Statement 6.
            let mut m = <THandler as GameHandler<TPosition>>::EVAL_MINIMUM;
            let mut pv = [None; SIZE];

            loop {
                // Statement 9.
                let (t, mut line) = self.branch_and_bound::<THandler, TPosition, SIZE>(
                    handler,
                    pos.play_move(mv),
                    depth - 1,
                    max_depth,
                    -m,
                );
                let t = -t;
                line[max_depth - depth] = Some(mv);

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
            (handler.evaluate(pos, depth, max_depth), [None; SIZE])
        }
    }

    // Algorithm B.
    pub fn alpha_beta<THandler, TPosition, const SIZE: usize>(
        &mut self,
        handler: &THandler,
        pos: TPosition,
        depth: usize,
        max_depth: usize,
        alpha: <THandler as GameHandler<TPosition>>::Eval,
        beta: <THandler as GameHandler<TPosition>>::Eval,
    ) -> MoveAndPV<THandler, TPosition, SIZE>
    where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        // A node `max_depth` plies ahead of the root is considered a leaf.
        // Statement 5.
        if depth == 0 {
            self.increment_leaf_count();
            return (handler.evaluate(pos, depth, max_depth), [None; SIZE]);
        }

        // Statement 4.
        let mut move_iter = handler.get_legal_moves(pos);

        if let Some(mut mv) = move_iter.next() {
            // Statement 6.
            let mut m = alpha;
            let mut pv = [None; SIZE];

            loop {
                // Statement 9.
                let (t, mut line) = self.alpha_beta::<THandler, TPosition, SIZE>(
                    handler,
                    pos.play_move(mv),
                    depth - 1,
                    max_depth,
                    -beta,
                    -m,
                );
                let t = -t;
                line[max_depth - depth] = Some(mv);

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
            (handler.evaluate(pos, depth, max_depth), [None; SIZE])
        }
    }

    // Algorithm C.
    pub fn p_alpha_beta<THandler, TPosition, const SIZE: usize>(
        &mut self,
        handler: &THandler,
        pos: TPosition,
        depth: usize,
        max_depth: usize,
    ) -> MoveAndPV<THandler, TPosition, SIZE>
    where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        // A node `max_depth` plies ahead of the root is considered a leaf.
        // Statement 5.
        if depth == 0 {
            self.increment_leaf_count();
            return (handler.evaluate(pos, depth, max_depth), [None; SIZE]);
        }

        // Statement 4.
        let mut move_iter = handler.get_legal_moves(pos);

        if let Some(mv) = move_iter.next() {
            // Statement 6.
            let (mut m, mut pv) = self.p_alpha_beta::<THandler, TPosition, SIZE>(
                handler,
                pos.play_move(mv),
                depth - 1,
                max_depth,
            );
            m = -m;
            pv[max_depth - depth] = Some(mv);

            let mut t: <THandler as GameHandler<TPosition>>::Eval;

            // Statement 7.
            for mv in move_iter {
                let next_pos = pos.play_move(mv);

                // Statement 9.
                t = -self.f_alpha_beta::<THandler, TPosition>(
                    handler,
                    next_pos,
                    depth - 1,
                    max_depth,
                    -m - <THandler as GameHandler<TPosition>>::EVAL_EPSILON,
                    -m,
                );

                // Statement 10.
                if t > m {
                    let (t, line) = self.alpha_beta::<THandler, TPosition, SIZE>(
                        handler,
                        next_pos,
                        depth - 1,
                        depth - 1,
                        <THandler as GameHandler<TPosition>>::EVAL_MINIMUM,
                        <THandler as GameHandler<TPosition>>::EVAL_MAXIMUM,
                    );
                    let t = -t;
                    if t > m {
                        m = t;
                        // Use alpha-beta with maximal window to retain PV without premature beta cutoffs.
                        // `-t` can be used in place of `EVAL_MAXIMUM` if PV does not need to be preserved.
                        // Time consumption increases due to the requirement of preserving PV.
                        // Fill in the PV with the shifted partial line returned from shallow alpha_beta call.
                        pv[max_depth - depth] = Some(mv);
                        for (i, &line_element) in (max_depth - depth + 1..SIZE).zip(line.iter()) {
                            pv[i] = line_element;
                        }
                    }
                }
            }

            (m, pv)
        } else {
            // Statement 5.
            self.increment_leaf_count();
            (handler.evaluate(pos, depth, max_depth), [None; SIZE])
        }
    }

    pub fn f_alpha_beta<THandler, TPosition>(
        &mut self,
        handler: &THandler,
        pos: TPosition,
        depth: usize,
        max_depth: usize,
        alpha: <THandler as GameHandler<TPosition>>::Eval,
        beta: <THandler as GameHandler<TPosition>>::Eval,
    ) -> <THandler as GameHandler<TPosition>>::Eval
    where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        // A node `max_depth` plies ahead of the root is considered a leaf.
        // Statement 5.
        if depth == 0 {
            self.increment_leaf_count();
            return handler.evaluate(pos, depth, max_depth);
        }

        // Statement 4.
        let mut move_iter = handler.get_legal_moves(pos);

        if let Some(mut mv) = move_iter.next() {
            // Statement 6.
            let mut m = <THandler as GameHandler<TPosition>>::EVAL_MINIMUM;

            loop {
                // Statement 9.
                m = std::cmp::max(
                    m,
                    -self.f_alpha_beta::<THandler, TPosition>(
                        handler,
                        pos.play_move(mv),
                        depth - 1,
                        max_depth,
                        -beta,
                        -std::cmp::max(m, alpha),
                    ),
                );

                // Statement 10.
                if m >= beta {
                    return m;
                }

                if let Some(new_mv) = move_iter.next() {
                    mv = new_mv;
                } else {
                    break;
                }
            }

            m
        } else {
            // Statement 5.
            self.increment_leaf_count();
            handler.evaluate(pos, depth, max_depth)
        }
    }

    // Algorithm D.
    pub fn pvs<THandler, TPosition, const SIZE: usize>(
        &mut self,
        handler: &THandler,
        pos: TPosition,
        depth: usize,
        max_depth: usize,
        alpha: <THandler as GameHandler<TPosition>>::Eval,
        beta: <THandler as GameHandler<TPosition>>::Eval,
    ) -> MoveAndPV<THandler, TPosition, SIZE>
    where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        // A node `max_depth` plies ahead of the root is considered a leaf.
        // Statement 5.
        if depth == 0 {
            self.increment_leaf_count();
            return (handler.evaluate(pos, depth, max_depth), [None; SIZE]);
        }

        // Statement 4.
        let mut move_iter = handler.get_legal_moves(pos);

        if let Some(mv) = move_iter.next() {
            // Statement 6.
            let (mut m, mut pv) = self.pvs::<THandler, TPosition, SIZE>(
                handler,
                pos.play_move(mv),
                depth - 1,
                max_depth,
                -beta,
                -alpha,
            );
            m = -m;
            pv[max_depth - depth] = Some(mv);

            // Statement 7.
            if m < beta {
                // Statement 8.
                for mv in move_iter {
                    // Statement 10.
                    let bound = std::cmp::max(m, alpha);

                    let next_pos = pos.play_move(mv);

                    // Statement 11.
                    let (t, mut line) = self.pvs::<THandler, TPosition, SIZE>(
                        handler,
                        next_pos,
                        depth - 1,
                        max_depth,
                        -bound - <THandler as GameHandler<TPosition>>::EVAL_EPSILON,
                        -bound,
                    );
                    let t = -t;
                    line[max_depth - depth] = Some(mv);

                    // Statement 12.
                    if t > m {
                        // Statement 13.
                        let (new_m, mut line) = self.pvs::<THandler, TPosition, SIZE>(
                            handler,
                            next_pos,
                            depth - 1,
                            max_depth,
                            -beta,
                            <THandler as GameHandler<TPosition>>::EVAL_MAXIMUM,
                        );
                        // Use pvs with maximal window to retain PV without premature beta cutoffs.
                        // `-t` can be used in place of `EVAL_MAXIMUM` if PV does not need to be preserved.
                        // Time consumption increases due to the requirement of preserving PV.
                        // Fill in the PV with the shifted partial line returned from shallow alpha_beta call.
                        let new_m = -new_m;
                        line[max_depth - depth] = Some(mv);
                        m = new_m;
                        pv = line;
                    }
                    // Statement 14.
                    if m >= beta {
                        return (m, line);
                    }
                }
            }

            (m, pv)
        } else {
            // Statement 5.
            self.increment_leaf_count();
            (handler.evaluate(pos, depth, max_depth), [None; SIZE])
        }
    }

    // Algorithm E.
    pub fn scout<THandler, TPosition, const SIZE: usize>(
        &mut self,
        handler: &THandler,
        pos: TPosition,
        depth: usize,
        max_depth: usize,
    ) -> MoveAndPV<THandler, TPosition, SIZE>
    where
        THandler: GameHandler<TPosition>,
        TPosition: GamePosition,
    {
        // A node `max_depth` plies ahead of the root is considered a leaf.
        // Statement 5.
        if depth == 0 {
            self.increment_leaf_count();
            return (handler.evaluate(pos, depth, max_depth), [None; SIZE]);
        }

        // Statement 4.
        let mut move_iter = handler.get_legal_moves(pos);

        if let Some(mv) = move_iter.next() {
            // Statement 6.
            let (mut m, mut pv) = self.scout::<THandler, TPosition, SIZE>(
                handler,
                pos.play_move(mv),
                depth - 1,
                max_depth,
            );
            m = -m;
            pv[max_depth - depth] = Some(mv);

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
                    max_depth,
                    -m,
                    !op,
                ) {
                    let (new_m, mut line) = self.scout::<THandler, TPosition, SIZE>(
                        handler,
                        next_pos,
                        depth - 1,
                        max_depth,
                    );
                    let new_m = -new_m;
                    line[max_depth - depth] = Some(mv);
                    m = new_m;
                    pv = line;
                }
            }

            (m, pv)
        } else {
            // Statement 5.
            self.increment_leaf_count();
            (handler.evaluate(pos, depth, max_depth), [None; SIZE])
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
    pub fn sss<THandler, TPosition, const SIZE: usize>(
        &mut self,
        handler: &THandler,
        root: TPosition,
        depth: usize,
        max_depth: usize,
    ) -> MoveAndPV<THandler, TPosition, SIZE>
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
                SIZE,
            >,
        > = BinaryHeap::new();

        open.push(State::Live {
            node: root,
            merit: (
                <THandler as GameHandler<TPosition>>::EVAL_MAXIMUM,
                [None; SIZE],
            ),
            depth,
            line: [None; SIZE],
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
                    if d == max_depth {
                        return (h, pv);
                    }
                    let mut parent = root;
                    let path_length = max_depth - d - 1;
                    for mv in l.iter().take(path_length) {
                        parent = parent.play_move(mv.unwrap());
                    }
                    if state.is_max_player(max_depth) {
                        if let Some(next_move) = handler
                            .get_legal_moves(parent)
                            .skip_while(|&mv| parent.play_move(mv) != n)
                            .nth(1)
                        {
                            l[path_length] = Some(next_move);
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
                        let eval = if ((max_depth - depth) & 1) == 0 {
                            handler.evaluate(n, depth, max_depth)
                        } else {
                            -handler.evaluate(n, depth, max_depth)
                        };
                        // Extension of Case 4. `max_depth` plies from root is considered leaf.
                        open.push(State::Solved {
                            node: n,
                            merit: if h < eval { (h, pv) } else { (eval, l) },
                            depth: d,
                            line: l,
                            iteration: i,
                        });
                    } else if let Some(first_move) = legal_moves.next() {
                        let mut line = l;
                        line[max_depth - d] = Some(first_move);
                        if state.is_max_player(max_depth) {
                            // Case 6.
                            open.push(State::Live {
                                node: n.play_move(first_move),
                                merit: (h, pv),
                                depth: d - 1,
                                line,
                                iteration: i,
                            });
                            for mv in legal_moves {
                                line[max_depth - d] = Some(mv);
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
                        let eval = if ((max_depth - depth) & 1) == 0 {
                            handler.evaluate(n, depth, max_depth)
                        } else {
                            -handler.evaluate(n, depth, max_depth)
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
