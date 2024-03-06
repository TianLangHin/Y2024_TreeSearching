use crate::prelude::*;
use std::collections::BinaryHeap;

// Replication of algorithms described in Muszycka & Shinghal (1985).

// Algorithm A.
pub fn branch_and_bound<THandler, TPosition>(
    handler: &THandler,
    pos: TPosition,
    depth: usize,
    max_depth: usize,
    bound: <THandler as GameHandler<TPosition>>::Eval,
) -> <THandler as GameHandler<TPosition>>::Eval
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition
{
    // A node `max_depth` plies ahead of the root is considered a leaf.
    // Statement 5.
    if depth == 0 {
        return handler.evaluate(pos);
    }

    // Statement 4.
    let mut move_iter = handler.get_legal_moves(pos);

    if let Some(mut mv) = move_iter.next() {

        // Statement 6.
        let mut m = <THandler as GameHandler<TPosition>>::EVAL_MINIMUM;

        loop {

            // Statement 9.
            let t = -branch_and_bound::<THandler, TPosition>(
                handler,
                pos.play_move(mv),
                depth - 1,
                max_depth,
                -m
            );
            if t > m {
                m = t;
            }

            // Statement 10.
            if m >= bound {
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
        handler.evaluate(pos)
    }
}

// Algorithm B.
pub fn alpha_beta<THandler, TPosition>(
    handler: &THandler,
    pos: TPosition,
    depth: usize,
    max_depth: usize,
    alpha: <THandler as GameHandler<TPosition>>::Eval,
    beta: <THandler as GameHandler<TPosition>>::Eval,
) -> <THandler as GameHandler<TPosition>>::Eval
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition
{
    // A node `max_depth` plies ahead of the root is considered a leaf.
    // Statement 5.
    if depth == 0 {
        return handler.evaluate(pos);
    }

    // Statement 4.
    let mut move_iter = handler.get_legal_moves(pos);

    if let Some(mut mv) = move_iter.next() {

        // Statement 6.
        let mut m = alpha;

        loop {

            // Statement 9.
            let t = -alpha_beta::<THandler, TPosition>(
                handler,
                pos.play_move(mv),
                depth - 1,
                max_depth,
                -beta,
                -m
            );
            if t > m {
                m = t;
            }

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
        handler.evaluate(pos)
    }
}

// Algorithm C.
pub fn p_alpha_beta<THandler, TPosition>(
    handler: &THandler,
    pos: TPosition,
    depth: usize,
    max_depth: usize,
) -> <THandler as GameHandler<TPosition>>::Eval
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition
{
    // A node `max_depth` plies ahead of the root is considered a leaf.
    // Statement 5.
    if depth == 0 {
        return handler.evaluate(pos);
    }

    // Statement 4.
    let mut move_iter = handler.get_legal_moves(pos);

    if let Some(mv) = move_iter.next() {

        // Statement 6.
        let mut m = -p_alpha_beta::<THandler, TPosition>(
            handler,
            pos.play_move(mv),
            depth - 1,
            max_depth
        );

        // Statement 7.
        while let Some(mv) = move_iter.next() {

            let next_pos = pos.play_move(mv);

            // Statement 9.
            let t = -f_alpha_beta::<THandler, TPosition>(
                handler,
                next_pos,
                depth - 1,
                max_depth,
                -m - <THandler as GameHandler<TPosition>>::EVAL_EPSILON,
                -m
            );

            // Statement 10.
            if t > m {
                m = -alpha_beta::<THandler, TPosition>(
                    handler,
                    next_pos,
                    depth - 1,
                    max_depth,
                    <THandler as GameHandler<TPosition>>::EVAL_MINIMUM,
                    -t
                );
            }
        }

        m
    } else {
        // Statement 5.
        handler.evaluate(pos)
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
    TPosition: GamePosition
{
    // A node `max_depth` plies ahead of the root is considered a leaf.
    // Statement 5.
    if depth == 0 {
        return handler.evaluate(pos);
    }

    // Statement 4.
    let mut move_iter = handler.get_legal_moves(pos);

    if let Some(mut mv) = move_iter.next() {

        // Statement 6.
        let mut m = <THandler as GameHandler<TPosition>>::EVAL_MINIMUM;

        loop {

            // Statement 9.
            let t = -f_alpha_beta::<THandler, TPosition>(
                handler,
                pos.play_move(mv),
                depth - 1,
                max_depth,
                -beta,
                -std::cmp::max(m, alpha)
            );
            if t > m {
                m = t;
            }

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
        handler.evaluate(pos)
    }
}

// Algorithm D.
pub fn pvs<THandler, TPosition>(
    handler: &THandler,
    pos: TPosition,
    depth: usize,
    max_depth: usize,
    alpha: <THandler as GameHandler<TPosition>>::Eval,
    beta: <THandler as GameHandler<TPosition>>::Eval,
) -> <THandler as GameHandler<TPosition>>::Eval
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition
{
    // A node `max_depth` plies ahead of the root is considered a leaf.
    // Statement 5.
    if depth == 0 {
        return handler.evaluate(pos);
    }

    // Statement 4.
    let mut move_iter = handler.get_legal_moves(pos);

    if let Some(mv) = move_iter.next() {

        // Statement 6.
        let mut m = -pvs::<THandler, TPosition>(
            handler,
            pos.play_move(mv),
            depth - 1,
            max_depth,
            -beta,
            -alpha,
        );

        // Statement 7.
        if m < beta {
            // Statement 8.
            while let Some(mv) = move_iter.next() {

                // Statement 10.
                let bound = std::cmp::max(m, alpha);

                let next_pos = pos.play_move(mv);

                // Statement 11.
                let t = -pvs::<THandler, TPosition>(
                    handler,
                    next_pos,
                    depth - 1,
                    max_depth,
                    -bound - <THandler as GameHandler<TPosition>>::EVAL_EPSILON,
                    -bound,
                );

                // Statement 12.
                if t > m {
                    // Statement 13.
                    m = -pvs::<THandler, TPosition>(
                        handler,
                        next_pos,
                        depth - 1,
                        max_depth,
                        -beta,
                        -t,
                    );
                }
                // Statement 14.
                if m >= beta {
                    return m;
                }
            }
        }

        m
    } else {
        // Statement 5.
        handler.evaluate(pos)
    }
}

// Algorithm E.
pub fn scout<THandler, TPosition>(
    handler: &THandler,
    pos: TPosition,
    depth: usize,
    max_depth: usize,
) -> <THandler as GameHandler<TPosition>>::Eval
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition
{
    // A node `max_depth` plies ahead of the root is considered a leaf.
    // Statement 5.
    if depth == 0 {
        return handler.evaluate(pos);
    }

    // Statement 4.
    let mut move_iter = handler.get_legal_moves(pos);

    if let Some(mv) = move_iter.next() {

        // Statement 6.
        let mut m = -scout::<THandler, TPosition>(
            handler,
            pos.play_move(mv),
            depth - 1,
            max_depth
        );

        // Statement 7.
        let op = true;

        // Statement 8.
        while let Some(mv) = move_iter.next() {

            let next_pos = pos.play_move(mv);

            // Statement 9.
            if !test::<THandler, TPosition>(handler, next_pos, depth - 1, max_depth, -m, !op) {
                m = -scout::<THandler, TPosition>(
                    handler,
                    next_pos,
                    depth - 1,
                    max_depth,
                );
            }
        }

        m
    } else {
        // Statement 5.
        handler.evaluate(pos)
    }
}

pub fn test<THandler, TPosition>(
    handler: &THandler,
    pos: TPosition,
    depth: usize,
    max_depth: usize,
    v: <THandler as GameHandler<TPosition>>::Eval,
    op: bool
) -> bool
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition
{
    // A node `max_depth` plies ahead of the root is considered a leaf.
    // Statement 5.
    if depth == 0 {
        // Statements 6-9.
        return if op { handler.evaluate(pos) >= v } else { handler.evaluate(pos) > v };
    }

    // Statement 4.
    let mut move_iter = handler.get_legal_moves(pos);

    if let Some(mut mv) = move_iter.next() {

        loop {

            // Statement 11.
            if !test::<THandler, TPosition>(handler, pos.play_move(mv), depth - 1, max_depth, -v, !op) {
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
        return if op { handler.evaluate(pos) >= v } else { handler.evaluate(pos) > v };
    }
}

// Algorithm F.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Live,
    Solved,
}

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

pub fn sss<H, P>(
    handler: &H,
    root: P,
    depth: usize,
    max_depth: usize,
) -> <H as GameHandler<P>>::Eval
where
    H: GameHandler<P>,
    P: GamePosition
{

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct State<TPos, TEval>
    where
        TPos: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
        TEval: Clone + Copy + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord
    {
        pub node: TPos,
        pub status: Status,
        pub node_type: NodeType,
        pub merit: TEval,
        pub depth: usize,
    }

    impl<TPos, TEval> PartialOrd for State<TPos, TEval>
    where
        TPos: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
        TEval: Clone + Copy + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord
    {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.merit.partial_cmp(&other.merit)
        }
    }

    impl<TPos, TEval> Ord for State<TPos, TEval>
    where
        TPos: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
        TEval: Clone + Copy + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord
    {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.merit.cmp(&other.merit)
        }
    }

    let mut parent_map: Vec<(P, P)> = Vec::new();

    let mut open: BinaryHeap<State<P, <H as GameHandler<P>>::Eval>> = BinaryHeap::new();
    open.push(
        State {
            node: root,
            status: Status::Live,
            node_type: NodeType::Max,
            merit: <H as GameHandler<P>>::EVAL_MAXIMUM,
            depth,
        }
    );

    let mut i: usize = 1;

    while let Some(State { node: n, status: s, node_type: nt, merit: h, depth: d }) = open.pop() {
        if d == max_depth && s == Status::Solved {
            println!("Iterations: {i}");
            return h;
        }
        if d != 0 {
            for mv in handler.get_legal_moves(n) {
                parent_map.push((n.play_move(mv), n));
            }
        }
        let mut legal_moves = handler.get_legal_moves(n);
        match s {
            Status::Solved => {
                let parent = parent_map
                    .iter()
                    .find(|&&(c, _)| c == n)
                    .expect("Only root does not have parent")
                    .1;
                match nt {
                    NodeType::Min => {
                        // Case 1.
                        let new_state = State {
                            node: parent,
                            status: Status::Solved,
                            node_type: nt.invert(),
                            merit: h,
                            depth: d + 1,
                        };
                        open.retain(|&state| {
                            let mut descendant = false;
                            let mut querying_node = state.node;
                            while let Some((_, p)) = parent_map
                                .iter()
                                .find(|&&(c, _)| c == querying_node)
                            {
                                if querying_node == new_state.node {
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
                            .skip(1)
                            .next()
                        {
                            // Case 2.
                            open.push(
                                State {
                                    node: parent.play_move(next_move),
                                    status: Status::Live,
                                    node_type: nt,
                                    merit: h,
                                    depth: d,
                                }
                            );
                        } else {
                            // Case 3.
                            open.push(
                                State {
                                    node: parent,
                                    status: Status::Solved,
                                    node_type: nt.invert(),
                                    merit: h,
                                    depth: d + 1,
                                }
                            );
                        }
                    }
                }
            }
            Status::Live => {
                if d == 0 {
                    // Extension of Case 4. `max_depth` plies from root is considered leaf.
                    open.push(
                        State {
                            node: n,
                            status: Status::Solved,
                            node_type: nt,
                            merit: std::cmp::min(h, handler.evaluate(n)),
                            depth: d,
                        }
                    );
                } else if let Some(first_move) = legal_moves.next() {
                    match nt {
                        NodeType::Min => {
                            // Case 5.
                            open.push(
                                State {
                                    node: n.play_move(first_move),
                                    status: Status::Live,
                                    node_type: nt.invert(),
                                    merit: h,
                                    depth: d - 1,
                                }
                            );
                        }
                        NodeType::Max => {
                            // Case 6.
                            open.push(
                                State {
                                    node: n.play_move(first_move),
                                    status: Status::Live,
                                    node_type: nt.invert(),
                                    merit: h,
                                    depth: d - 1,
                                }
                            );
                            while let Some(mv) = legal_moves.next() {
                                open.push(
                                    State {
                                        node: n.play_move(mv),
                                        status: Status::Live,
                                        node_type: nt.invert(),
                                        merit: h,
                                        depth: d - 1,
                                    }
                                );
                            }
                        }
                    }
                } else {
                    // Next legal move is `None` on first attempt: leaf node. Thus, Case 4.
                    open.push(
                        State {
                            node: n,
                            status: Status::Solved,
                            node_type: nt,
                            merit: std::cmp::min(h, handler.evaluate(n)),
                            depth: d,
                        }
                    );
                }
            }
        }
        i += 1;
    }
    panic!("State space operator is faulty");
}