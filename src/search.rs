use crate::prelude::*;
use std::collections::BinaryHeap;

pub type MoveAndPV<THandler, TPosition, const SIZE: usize> = (
    <THandler as GameHandler<TPosition>>::Eval,
    [Option<<TPosition as GamePosition>::Move>; SIZE],
);

// Replication of algorithms described in Muszycka & Shinghal (1985).

// Algorithm A.
pub fn branch_and_bound<THandler, TPosition, const SIZE: usize>(
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
            let (t, mut line) = branch_and_bound::<THandler, TPosition, SIZE>(
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
        (handler.evaluate(pos, depth, max_depth), [None; SIZE])
    }
}

// Algorithm B.
pub fn alpha_beta<THandler, TPosition, const SIZE: usize>(
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
            let (t, mut line) = alpha_beta::<THandler, TPosition, SIZE>(
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
        (handler.evaluate(pos, depth, max_depth), [None; SIZE])
    }
}

// Algorithm C.
pub fn p_alpha_beta<THandler, TPosition, const SIZE: usize>(
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
        return (handler.evaluate(pos, depth, max_depth), [None; SIZE]);
    }

    // Statement 4.
    let mut move_iter = handler.get_legal_moves(pos);

    if let Some(mv) = move_iter.next() {
        // Statement 6.
        let (mut m, mut pv) = p_alpha_beta::<THandler, TPosition, SIZE>(
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
            t = -f_alpha_beta::<THandler, TPosition>(
                handler,
                next_pos,
                depth - 1,
                max_depth,
                -m - <THandler as GameHandler<TPosition>>::EVAL_EPSILON,
                -m,
            );

            // Statement 10.
            if t > m {
                let (t, line) = alpha_beta::<THandler, TPosition, SIZE>(
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
        (handler.evaluate(pos, depth, max_depth), [None; SIZE])
    }
}

pub fn f_alpha_beta<THandler, TPosition>(
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
                -f_alpha_beta::<THandler, TPosition>(
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
        handler.evaluate(pos, depth, max_depth)
    }
}

// Algorithm D.
pub fn pvs<THandler, TPosition, const SIZE: usize>(
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
        return (handler.evaluate(pos, depth, max_depth), [None; SIZE]);
    }

    // Statement 4.
    let mut move_iter = handler.get_legal_moves(pos);

    if let Some(mv) = move_iter.next() {
        // Statement 6.
        let (mut m, mut pv) = pvs::<THandler, TPosition, SIZE>(
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
                let (t, mut line) = pvs::<THandler, TPosition, SIZE>(
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
                    let (new_m, mut line) = pvs::<THandler, TPosition, SIZE>(
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
        (handler.evaluate(pos, depth, max_depth), [None; SIZE])
    }
}

// Algorithm E.
pub fn scout<THandler, TPosition, const SIZE: usize>(
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
        return (handler.evaluate(pos, depth, max_depth), [None; SIZE]);
    }

    // Statement 4.
    let mut move_iter = handler.get_legal_moves(pos);

    if let Some(mv) = move_iter.next() {
        // Statement 6.
        let (mut m, mut pv) =
            scout::<THandler, TPosition, SIZE>(handler, pos.play_move(mv), depth - 1, max_depth);
        m = -m;
        pv[max_depth - depth] = Some(mv);

        // Statement 7.
        let op = true;

        // Statement 8.
        for mv in move_iter {
            let next_pos = pos.play_move(mv);

            // Statement 9.
            if !test::<THandler, TPosition>(handler, next_pos, depth - 1, max_depth, -m, !op) {
                let (new_m, mut line) =
                    scout::<THandler, TPosition, SIZE>(handler, next_pos, depth - 1, max_depth);
                let new_m = -new_m;
                line[max_depth - depth] = Some(mv);
                m = new_m;
                pv = line;
            }
        }

        (m, pv)
    } else {
        // Statement 5.
        (handler.evaluate(pos, depth, max_depth), [None; SIZE])
    }
}

pub fn test<THandler, TPosition>(
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
            if !test::<THandler, TPosition>(
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
        if op {
            handler.evaluate(pos, depth, max_depth) >= v
        } else {
            handler.evaluate(pos, depth, max_depth) > v
        }
    }
}

// Algorithm F.
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
    },
    Solved {
        node: TPos,
        merit: (TEval, [Option<TMove>; SIZE]),
        depth: usize,
        line: [Option<TMove>; SIZE],
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
            } => merit,
            Self::Live {
                node: _,
                merit,
                depth: _,
                line: _,
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
            } => depth,
            Self::Live {
                node: _,
                merit: _,
                depth,
                line: _,
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
            } => line,
            Self::Live {
                node: _,
                merit: _,
                depth: _,
                line,
            } => line,
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
        self.merit().0.cmp(&other.merit().0)
    }
}

pub fn sss<THandler, TPosition, const SIZE: usize>(
    handler: &THandler,
    root: TPosition,
    depth: usize,
    max_depth: usize,
) -> MoveAndPV<THandler, TPosition, SIZE>
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
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
    });

    while let Some(state) = open.pop() {
        match state {
            State::Solved {
                node: n,
                merit: (h, pv),
                depth: d,
                line: mut l,
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
                        });
                    } else {
                        // Case 3.
                        open.push(State::Solved {
                            node: parent,
                            merit: (h, pv),
                            depth: d + 1,
                            line: l,
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
                    });
                }
            }
            State::Live {
                node: n,
                merit: (h, pv),
                depth: d,
                line: l,
            } => {
                let mut legal_moves = handler.get_legal_moves(n);
                if d == 0 {
                    // To account for the negamax construct in conjunction with SSS* node evaluation.
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
                        });
                        for mv in legal_moves {
                            line[max_depth - d] = Some(mv);
                            open.push(State::Live {
                                node: n.play_move(mv),
                                merit: (h, pv),
                                depth: d - 1,
                                line,
                            });
                        }
                    } else {
                        // Case 5.
                        open.push(State::Live {
                            node: n.play_move(first_move),
                            merit: (h, pv),
                            depth: d - 1,
                            line,
                        });
                    }
                } else {
                    // To account for the negamax construct in conjunction with SSS* node evaluation.
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
                    });
                }
            }
        }
        // println!("{:?}", &open);
    }
    panic!("State space operator is faulty");
}
