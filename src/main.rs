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

use colored::Colorize;
use rayon::prelude::*;
use seq_macro::seq;

use std::time::Instant;

// Depth-Width pair for setting each hypothetical tree configuration.
type TreeSetting = (usize, usize);

// The typing for the list of settings to be tested,
// but arranged as a tuple since this is required by const-ness in seq_macro.
type AllTreeSettings = (
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
    TreeSetting,
);

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
            println!(
                "{}: {num}",
                handler.move_string(mv, (pos.squares >> 19) & 1)
            );
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
            println!(
                "{}: {num}",
                handler.move_string(mv, (pos.squares >> 19) & 1)
            );
            num
        })
        .sum();
    println!("Nodes searched: {sum}");
    println!("Time elapsed: {} ms", s.elapsed().as_millis());
}

fn eval_from_line<THandler, TPosition, const SIZE: usize>(
    handler: &THandler,
    initial_pos: TPosition,
    line: [Option<<TPosition as GamePosition>::Move>; SIZE],
) -> <THandler as GameHandler<TPosition>>::Eval
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    let mut pos = initial_pos;
    let mut depth = 0;
    for &mv in line.iter() {
        if let Some(m) = mv {
            pos = pos.play_move(m);
            depth += 1;
        }
    }
    if (depth & 1) == 0 {
        handler.evaluate(pos, SIZE - depth, SIZE)
    } else {
        -handler.evaluate(pos, SIZE - depth, SIZE)
    }
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
    p_alpha_beta(handler, root, DEPTH, DEPTH)
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
    scout(handler, root, DEPTH, DEPTH)
}

fn root_call_sss<THandler, TPosition, const DEPTH: usize>(
    handler: &THandler,
    root: TPosition,
) -> MoveAndPV<THandler, TPosition, DEPTH>
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    sss(handler, root, DEPTH, DEPTH)
}

fn raw_moves_display<T, const SIZE: usize>(
    move_list: [Option<T>; SIZE]
) -> String
where
    T: std::fmt::Debug
{
    move_list
        .iter()
        .map_while(|mv| mv.as_ref().map(|m| format!("{:?}", m)))
        .collect::<Vec<String>>()
        .join(", ")
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

    let mut results: Vec<MoveAndPV<THandler, TPosition, DEPTH>> = Vec::new();

    println!("{}", position_name.bright_magenta());
    seq!(N in 0..6 {
        println!("{}", algorithm_names.N.bright_cyan());
        let s = Instant::now();
        let result: MoveAndPV<THandler, TPosition, DEPTH> = algorithms.N(&handler, startpos);
        println!("Time elapsed: {} ms", s.elapsed().as_millis().to_string().bright_yellow());
        let recalculated_eval = eval_from_line(&handler, startpos, result.1);
        if recalculated_eval == result.0 {
            println!("Eval and Line {}", "MATCH".bright_green());
            println!(
                "Eval: {}, Line: {}",
                format!("{:?}", result.0).bright_green(),
                raw_moves_display(result.1).bright_green(),
            );
        } else {
            println!("Eval and Line {}", "MISMATCH".bright_red());
            println!(
                "Expected Eval: {}, Got Eval: {}",
                format!("{:?}", result.0).bright_green(),
                format!("{:?}", recalculated_eval).bright_red(),
            );
        }
        results.push(result);
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

    const DEPTH_WIDTH_PAIRS: AllTreeSettings = (
        (2, 2),
        (2, 3),
        (2, 4),
        (2, 5),
        (2, 6),
        (2, 8),
        (2, 10),
        (2, 24),
        (3, 2),
        (3, 3),
        (3, 4),
        (3, 5),
        (3, 6),
        (3, 8),
        (3, 10),
        (4, 2),
        (4, 3),
        (4, 4),
        (4, 5),
        (5, 2),
        (5, 3),
        (5, 4),
        (6, 2),
        (6, 3),
    );

    seq!(N in 0..24 {
        test_algorithms::<UnordIndHypTreeHandler, HypTreePos, { DEPTH_WIDTH_PAIRS.N.0 }>(
            &format!(
                "Unordered-Independent Hypothetical Game Tree (Depth = {}, Width = {})",
                DEPTH_WIDTH_PAIRS.N.0,
                DEPTH_WIDTH_PAIRS.N.1,
            ),
            HypTreeParams {
                depth: DEPTH_WIDTH_PAIRS.N.0,
                width: DEPTH_WIDTH_PAIRS.N.1,
                seed: 314159,
            },
            8,
        );
    });
}
