use crate::prelude::*;
use std::collections::BinaryHeap;

pub type MoveAndPV<THandler, TPosition, TParams, const SIZE: usize> = (
    <THandler as GameHandler<TPosition, TParams>>::Eval,
    [Option<<TPosition as GamePosition>::Move>; SIZE],
);

// Replication of algorithms described in Muszycka & Shinghal (1985).

// Algorithm A.
pub fn branch_and_bound<THandler, TPosition, TParams, const SIZE: usize>(
    handler: &THandler,
    pos: TPosition,
    depth: usize,
    max_depth: usize,
    bound: <THandler as GameHandler<TPosition, TParams>>::Eval,
) -> MoveAndPV<THandler, TPosition, TParams, SIZE>
where
    THandler: GameHandler<TPosition, TParams>,
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
        let mut m = <THandler as GameHandler<TPosition, TParams>>::EVAL_MINIMUM;
        let mut pv = [None; SIZE];

        loop {
            // Statement 9.
            let (t, mut line) = branch_and_bound::<THandler, TPosition, TParams, SIZE>(
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
pub fn alpha_beta<THandler, TPosition, TParams, const SIZE: usize>(
    handler: &THandler,
    pos: TPosition,
    depth: usize,
    max_depth: usize,
    alpha: <THandler as GameHandler<TPosition, TParams>>::Eval,
    beta: <THandler as GameHandler<TPosition, TParams>>::Eval,
) -> MoveAndPV<THandler, TPosition, TParams, SIZE>
where
    THandler: GameHandler<TPosition, TParams>,
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
            let (t, mut line) = alpha_beta::<THandler, TPosition, TParams, SIZE>(
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
pub fn p_alpha_beta<THandler, TPosition, TParams, const SIZE: usize>(
    handler: &THandler,
    pos: TPosition,
    depth: usize,
    max_depth: usize,
) -> MoveAndPV<THandler, TPosition, TParams, SIZE>
where
    THandler: GameHandler<TPosition, TParams>,
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
        let (mut m, mut pv) = p_alpha_beta::<THandler, TPosition, TParams, SIZE>(
            handler,
            pos.play_move(mv),
            depth - 1,
            max_depth,
        );
        m = -m;
        pv[max_depth - depth] = Some(mv);

        // Statement 7.
        for mv in move_iter {
            let next_pos = pos.play_move(mv);

            // Statement 9.
            let (t, mut line) = f_alpha_beta::<THandler, TPosition, TParams, SIZE>(
                handler,
                next_pos,
                depth - 1,
                max_depth,
                -m - <THandler as GameHandler<TPosition, TParams>>::EVAL_EPSILON,
                -m,
            );
            let t = -t;
            line[max_depth - depth] = Some(mv);

            // Statement 10.
            if t > m {
                m = -(alpha_beta::<THandler, TPosition, TParams, SIZE>(
                    handler,
                    next_pos,
                    depth - 1,
                    max_depth,
                    <THandler as GameHandler<TPosition, TParams>>::EVAL_MINIMUM,
                    -t,
                )
                .0);
                pv = line;
            }
        }

        (m, pv)
    } else {
        // Statement 5.
        (handler.evaluate(pos, depth, max_depth), [None; SIZE])
    }
}

pub fn f_alpha_beta<THandler, TPosition, TParams, const SIZE: usize>(
    handler: &THandler,
    pos: TPosition,
    depth: usize,
    max_depth: usize,
    alpha: <THandler as GameHandler<TPosition, TParams>>::Eval,
    beta: <THandler as GameHandler<TPosition, TParams>>::Eval,
) -> MoveAndPV<THandler, TPosition, TParams, SIZE>
where
    THandler: GameHandler<TPosition, TParams>,
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
        let mut m = <THandler as GameHandler<TPosition, TParams>>::EVAL_MINIMUM;
        let mut pv = [None; SIZE];

        loop {
            // Statement 9.
            let (t, mut line) = f_alpha_beta::<THandler, TPosition, TParams, SIZE>(
                handler,
                pos.play_move(mv),
                depth - 1,
                max_depth,
                -beta,
                -std::cmp::max(m, alpha),
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

// Algorithm D.
pub fn pvs<THandler, TPosition, TParams, const SIZE: usize>(
    handler: &THandler,
    pos: TPosition,
    depth: usize,
    max_depth: usize,
    alpha: <THandler as GameHandler<TPosition, TParams>>::Eval,
    beta: <THandler as GameHandler<TPosition, TParams>>::Eval,
) -> MoveAndPV<THandler, TPosition, TParams, SIZE>
where
    THandler: GameHandler<TPosition, TParams>,
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
        let (mut m, mut pv) = pvs::<THandler, TPosition, TParams, SIZE>(
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
                let (t, mut line) = pvs::<THandler, TPosition, TParams, SIZE>(
                    handler,
                    next_pos,
                    depth - 1,
                    max_depth,
                    -bound - <THandler as GameHandler<TPosition, TParams>>::EVAL_EPSILON,
                    -bound,
                );
                let t = -t;
                line[max_depth - depth] = Some(mv);

                // Statement 12.
                if t > m {
                    // Statement 13.
                    let (new_m, mut line) = pvs::<THandler, TPosition, TParams, SIZE>(
                        handler,
                        next_pos,
                        depth - 1,
                        max_depth,
                        -beta,
                        -t,
                    );
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
pub fn scout<THandler, TPosition, TParams, const SIZE: usize>(
    handler: &THandler,
    pos: TPosition,
    depth: usize,
    max_depth: usize,
) -> MoveAndPV<THandler, TPosition, TParams, SIZE>
where
    THandler: GameHandler<TPosition, TParams>,
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
        let (mut m, mut pv) = scout::<THandler, TPosition, TParams, SIZE>(
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
            if !test::<THandler, TPosition, TParams>(
                handler,
                next_pos,
                depth - 1,
                max_depth,
                -m,
                !op,
            ) {
                let (new_m, mut line) = scout::<THandler, TPosition, TParams, SIZE>(
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
        (handler.evaluate(pos, depth, max_depth), [None; SIZE])
    }
}

pub fn test<THandler, TPosition, TParams>(
    handler: &THandler,
    pos: TPosition,
    depth: usize,
    max_depth: usize,
    v: <THandler as GameHandler<TPosition, TParams>>::Eval,
    op: bool,
) -> bool
where
    THandler: GameHandler<TPosition, TParams>,
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
            if !test::<THandler, TPosition, TParams>(
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeType {
    Max,
    Min,
}

impl NodeType {
    fn invert(&self) -> Self {
        match *self {
            Self::Max => Self::Min,
            Self::Min => Self::Max,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State<TPos, TEval>
where
    TPos: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
    TEval: Clone + Copy + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord,
{
    Live {
        node: TPos,
        node_type: NodeType,
        merit: TEval,
        depth: usize,
    },
    Solved {
        node: TPos,
        node_type: NodeType,
        merit: TEval,
        depth: usize,
    },
}

impl<TPos, TEval> PartialOrd for State<TPos, TEval>
where
    TPos: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
    TEval: Clone + Copy + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<TPos, TEval> Ord for State<TPos, TEval>
where
    TPos: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
    TEval: Clone + Copy + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_merit = match self {
            Self::Solved {
                node: _,
                node_type: _,
                merit: m,
                depth: _,
            } => m,
            Self::Live {
                node: _,
                node_type: _,
                merit: m,
                depth: _,
            } => m,
        };
        let other_merit = match other {
            Self::Solved {
                node: _,
                node_type: _,
                merit: m,
                depth: _,
            } => m,
            Self::Live {
                node: _,
                node_type: _,
                merit: m,
                depth: _,
            } => m,
        };
        self_merit.cmp(other_merit)
    }
}

pub fn sss<THandler, TPosition, TParams>(
    handler: &THandler,
    root: TPosition,
    depth: usize,
    max_depth: usize,
) -> <THandler as GameHandler<TPosition, TParams>>::Eval
where
    THandler: GameHandler<TPosition, TParams>,
    TPosition: GamePosition,
{
    let mut parent_map: Vec<(TPosition, TPosition)> = Vec::new();

    let mut open: BinaryHeap<
        State<TPosition, <THandler as GameHandler<TPosition, TParams>>::Eval>,
    > = BinaryHeap::new();
    open.push(State::Live {
        node: root,
        node_type: NodeType::Max,
        merit: <THandler as GameHandler<TPosition, TParams>>::EVAL_MAXIMUM,
        depth,
    });

    while let Some(state) = open.pop() {
        match state {
            State::Solved {
                node: n,
                node_type: nt,
                merit: h,
                depth: d,
            } => {
                if d == max_depth {
                    return h;
                }
                if d != 0 {
                    for mv in handler.get_legal_moves(n) {
                        parent_map.push((n.play_move(mv), n));
                    }
                }
                let parent = parent_map
                    .iter()
                    .find(|&&(c, _)| c == n)
                    .expect("Only root does not have parent")
                    .1;
                match nt {
                    NodeType::Min => {
                        // Case 1.
                        let new_state = State::Solved {
                            node: parent,
                            node_type: nt.invert(),
                            merit: h,
                            depth: d + 1,
                        };
                        open.retain(|&state| {
                            let mut descendant = false;
                            let mut querying_node = match state {
                                State::Solved {
                                    node: n,
                                    node_type: _,
                                    merit: _,
                                    depth: _,
                                } => n,
                                State::Live {
                                    node: n,
                                    node_type: _,
                                    merit: _,
                                    depth: _,
                                } => n,
                            };
                            while let Some((_, p)) =
                                parent_map.iter().find(|&&(c, _)| c == querying_node)
                            {
                                if querying_node == parent {
                                    descendant = true;
                                    break;
                                }
                                querying_node = *p;
                            }
                            !descendant
                        });
                        open.push(new_state);
                    }
                    NodeType::Max => {
                        if let Some(next_move) = handler
                            .get_legal_moves(parent)
                            .skip_while(|&mv| parent.play_move(mv) != n)
                            .nth(1)
                        {
                            // Case 2.
                            open.push(State::Live {
                                node: parent.play_move(next_move),
                                node_type: nt,
                                merit: h,
                                depth: d,
                            });
                        } else {
                            // Case 3.
                            open.push(State::Solved {
                                node: parent,
                                node_type: nt.invert(),
                                merit: h,
                                depth: d + 1,
                            });
                        }
                    }
                }
            }
            State::Live {
                node: n,
                node_type: nt,
                merit: h,
                depth: d,
            } => {
                let mut legal_moves = handler.get_legal_moves(n);
                if d == 0 {
                    // Extension of Case 4. `max_depth` plies from root is considered leaf.
                    open.push(State::Solved {
                        node: n,
                        node_type: nt,
                        merit: std::cmp::min(h, handler.evaluate(n, depth, max_depth)),
                        depth: d,
                    });
                } else if let Some(first_move) = legal_moves.next() {
                    parent_map.push((n.play_move(first_move), n));
                    match nt {
                        NodeType::Min => {
                            // Case 5.
                            open.push(State::Live {
                                node: n.play_move(first_move),
                                node_type: nt.invert(),
                                merit: h,
                                depth: d - 1,
                            });
                            for mv in legal_moves {
                                parent_map.push((n.play_move(mv), n));
                            }
                        }
                        NodeType::Max => {
                            // Case 6.
                            open.push(State::Live {
                                node: n.play_move(first_move),
                                node_type: nt.invert(),
                                merit: h,
                                depth: d - 1,
                            });
                            for mv in legal_moves {
                                parent_map.push((n.play_move(mv), n));
                                open.push(State::Live {
                                    node: n.play_move(mv),
                                    node_type: nt.invert(),
                                    merit: h,
                                    depth: d - 1,
                                });
                            }
                        }
                    }
                } else {
                    // Next legal move is `None` on first attempt: leaf node. Thus, Case 4.
                    open.push(State::Solved {
                        node: n,
                        node_type: nt,
                        merit: std::cmp::min(h, handler.evaluate(n, depth, max_depth)),
                        depth: d,
                    });
                }
            }
        }
    }
    panic!("State space operator is faulty");
}
