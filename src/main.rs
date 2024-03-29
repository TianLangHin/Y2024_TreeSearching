use crate::games::chess::*;
use crate::games::hypothetical_tree::*;
use crate::games::stockman::*;
use crate::games::uniform_2b_wide::*;
use crate::games::ut3::*;
use crate::prelude::*;
use crate::search::*;

pub mod games;
pub mod prelude;
pub mod search;

use rayon::prelude::*;
use seq_macro::seq;
use colored::Colorize;

use std::time::Instant;

fn test_stockman_tree() {
    let handler = StockmanHandler::new(());
    let mut nodes = vec![StockmanPos::startpos(())];
    while !nodes.is_empty() {
        dbg!(&nodes);
        let new_nodes = nodes
            .clone()
            .iter()
            .flat_map(|&node| {
                handler
                    .get_legal_moves(node)
                    .map(|mv| node.play_move(mv))
                    .collect::<Vec<_>>()
            })
            .collect();
        nodes = new_nodes;
    }
}

fn perft(depth: usize, pos: ChessPos, handler: &ChessHandler) -> u64 {
    if depth == 1 {
        return handler.get_legal_moves(pos).count() as u64;
    }
    handler
        .get_legal_moves(pos)
        .map(|mv| perft(depth - 1, pos.play_move(mv), handler))
        .sum()
}

fn perft_div_main(depth: usize, pos: ChessPos, handler: &ChessHandler) {
    println!("Serial perft");
    if depth == 1 {
        let s = Instant::now();
        println!("Nodes searched: {}", handler.get_legal_moves(pos).count());
        println!("Time elapsed: {} ms", s.elapsed().as_millis());
        return;
    }
    let s = Instant::now();
    let sum: u64 = handler
        .get_legal_moves(pos)
        .map(|mv| {
            let num = perft(depth - 1, pos.play_move(mv), handler);
            println!("{}: {num}", handler.move_string(mv, (pos.squares >> 19) & 1));
            num
        })
        .sum();
    println!("Nodes searched: {sum}");
    println!("Time elapsed: {} ms", s.elapsed().as_millis());
}

fn perft_div_main_par(depth: usize, pos: ChessPos, handler: &ChessHandler) {
    println!("Parallel perft");
    if depth == 1 {
        let s = Instant::now();
        println!("Nodes searched: {}", handler.get_legal_moves(pos).count());
        println!("Time elapsed: {} ms", s.elapsed().as_millis());
        return;
    }
    let s = Instant::now();
    let sum: u64 = handler
        .get_legal_moves(pos)
        .collect::<Vec<_>>()
        .par_iter()
        .map(|&mv| {
            let num = perft(depth - 1, pos.play_move(mv), handler);
            println!("{}: {num}", handler.move_string(mv, (pos.squares >> 19) & 1));
            num
        })
        .sum();
    println!("Nodes searched: {sum}");
    println!("Time elapsed: {} ms", s.elapsed().as_millis());
}

fn eval_from_line<THandler, TPosition, const SIZE: usize>(
    handler: &THandler,
    initial_pos: TPosition,
    line: [Option<<TPosition as GamePosition>::Move>; SIZE]
) -> <THandler as GameHandler<TPosition>>::Eval
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    let mut pos = initial_pos;
    let mut depth = SIZE;
    for &mv in line.iter() {
        if let Some(m) = mv {
            pos = pos.play_move(m);
            depth -= 1;
        }
    }
    handler.evaluate(pos, depth, SIZE)
}

fn root_call_bb<THandler, TPosition, const DEPTH: usize>(
    handler: &THandler,
    root: TPosition,
) -> MoveAndPV<THandler, TPosition, DEPTH>
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    branch_and_bound(
        handler,
        root,
        DEPTH,
        DEPTH,
        <THandler as GameHandler<TPosition>>::EVAL_MAXIMUM,
    )
}

fn root_call_ab<THandler, TPosition, const DEPTH: usize>(
    handler: &THandler,
    root: TPosition,
) -> MoveAndPV<THandler, TPosition, DEPTH>
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    alpha_beta(
        handler,
        root,
        DEPTH,
        DEPTH,
        <THandler as GameHandler<TPosition>>::EVAL_MINIMUM,
        <THandler as GameHandler<TPosition>>::EVAL_MAXIMUM,
    )
}

fn root_call_pab<THandler, TPosition, const DEPTH: usize>(
    handler: &THandler,
    root: TPosition,
) -> MoveAndPV<THandler, TPosition, DEPTH>
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    p_alpha_beta(
        handler,
        root,
        DEPTH,
        DEPTH,
    )
}

fn root_call_pvs<THandler, TPosition, const DEPTH: usize>(
    handler: &THandler,
    root: TPosition,
) -> MoveAndPV<THandler, TPosition, DEPTH>
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    pvs(
        handler,
        root,
        DEPTH,
        DEPTH,
        <THandler as GameHandler<TPosition>>::EVAL_MINIMUM,
        <THandler as GameHandler<TPosition>>::EVAL_MAXIMUM,
    )
}

fn root_call_scout<THandler, TPosition, const DEPTH: usize>(
    handler: &THandler,
    root: TPosition,
) -> MoveAndPV<THandler, TPosition, DEPTH>
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    scout(
        handler,
        root,
        DEPTH,
        DEPTH,
    )
}

fn root_call_sss<THandler, TPosition, const DEPTH: usize>(
    handler: &THandler,
    root: TPosition,
) -> MoveAndPV<THandler, TPosition, DEPTH>
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    sss(
        handler,
        root,
        DEPTH,
        DEPTH,
    )
}

fn test_algorithms<THandler, TPosition, const DEPTH: usize>(
    position_name: &str,
    handler_params: <THandler as GameHandler<TPosition>>::Params,
    startpos_params: <TPosition as GamePosition>::Params,
)
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    let handler = <THandler as GameHandler<TPosition>>::new(handler_params);
    let startpos = <TPosition as GamePosition>::startpos(startpos_params);

    let algorithms = (
        root_call_bb,
        root_call_ab,
        root_call_pab,
        root_call_pvs,
        root_call_scout,
        root_call_sss,
    );

    let algorithm_names = (
        "branch_and_bound",
        "alpha_beta",
        "p_alpha_beta",
        "pvs",
        "scout",
        "sss",
    );

    println!("{}", position_name.bright_magenta());
    seq!(N in 0..6 {
        println!("{}", algorithm_names.N.bright_cyan());
        let s = Instant::now();
        let result: MoveAndPV<THandler, TPosition, DEPTH> = algorithms.N(&handler, startpos);
        println!("Time elapsed: {} ms", s.elapsed().as_millis().to_string().bright_yellow());
        if eval_from_line(&handler, startpos, result.1) == result.0 {
            println!("Line and eval {}", "MATCH".bright_green());
        } else {
            println!("Line and eval {}", "MISMATCH".bright_red());
        }
    });
}

fn main() {
    test_algorithms::<StockmanHandler, StockmanPos, 4>("Stockman, G.C. (1979)", (), ());
    test_algorithms::<Ut3Handler, Ut3Board, 6>("Ultimate Tic-Tac-Toe", (), ());
    test_algorithms::<Uniform2bWideHandler, Uniform2bWidePos, 16>(
        "Uniform Tree (Branching Factor = 2)",
        Uniform2bWideParams {
            depth: 16,
            seed: 314159,
        },
        (),
    );
    test_algorithms::<UnordIndHypTreeHandler, HypTreePos, 8>(
        "Unordered-Independent Hypothetical Game Tree (Depth = 8, Width = 8)",
        HypTreeParams {
            depth: 8,
            width: 8,
            seed: 314159,
        },
        8,
    );


}
