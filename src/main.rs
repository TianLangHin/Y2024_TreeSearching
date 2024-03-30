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

#[derive(Clone, Copy, Debug)]
struct AlgorithmStats {
    pub avg_leaves: f64,
    pub avg_ms: f64,
    pub avg_us: f64,
    pub avg_ns: f64,
}

impl AlgorithmStats {
    fn new() -> Self {
        Self {
            avg_leaves: 0.0,
            avg_ms: 0.0,
            avg_us: 0.0,
            avg_ns: 0.0,
        }
    }
}

impl Default for AlgorithmStats {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Add for AlgorithmStats {
    type Output = AlgorithmStats;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            avg_leaves: self.avg_leaves + rhs.avg_leaves,
            avg_ms: self.avg_ms + rhs.avg_ms,
            avg_us: self.avg_us + rhs.avg_us,
            avg_ns: self.avg_ns + rhs.avg_ns,
        }
    }
}

impl std::ops::AddAssign for AlgorithmStats {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

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
    searcher: &mut Searcher,
    handler: &THandler,
    root: TPosition,
) -> MoveAndPV<THandler, TPosition, DEPTH>
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    searcher.branch_and_bound(
        handler,
        root,
        DEPTH,
        DEPTH,
        <THandler as GameHandler<TPosition>>::EVAL_MAXIMUM,
    )
}

fn root_call_ab<THandler, TPosition, const DEPTH: usize>(
    searcher: &mut Searcher,
    handler: &THandler,
    root: TPosition,
) -> MoveAndPV<THandler, TPosition, DEPTH>
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    searcher.alpha_beta(
        handler,
        root,
        DEPTH,
        DEPTH,
        <THandler as GameHandler<TPosition>>::EVAL_MINIMUM,
        <THandler as GameHandler<TPosition>>::EVAL_MAXIMUM,
    )
}

fn root_call_pab<THandler, TPosition, const DEPTH: usize>(
    searcher: &mut Searcher,
    handler: &THandler,
    root: TPosition,
) -> MoveAndPV<THandler, TPosition, DEPTH>
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    searcher.p_alpha_beta(handler, root, DEPTH, DEPTH)
}

fn root_call_pvs<THandler, TPosition, const DEPTH: usize>(
    searcher: &mut Searcher,
    handler: &THandler,
    root: TPosition,
) -> MoveAndPV<THandler, TPosition, DEPTH>
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    searcher.pvs(
        handler,
        root,
        DEPTH,
        DEPTH,
        <THandler as GameHandler<TPosition>>::EVAL_MINIMUM,
        <THandler as GameHandler<TPosition>>::EVAL_MAXIMUM,
    )
}

fn root_call_scout<THandler, TPosition, const DEPTH: usize>(
    searcher: &mut Searcher,
    handler: &THandler,
    root: TPosition,
) -> MoveAndPV<THandler, TPosition, DEPTH>
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    searcher.scout(handler, root, DEPTH, DEPTH)
}

fn root_call_sss<THandler, TPosition, const DEPTH: usize>(
    searcher: &mut Searcher,
    handler: &THandler,
    root: TPosition,
) -> MoveAndPV<THandler, TPosition, DEPTH>
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    searcher.sss(handler, root, DEPTH, DEPTH)
}

fn raw_moves_display<T, const SIZE: usize>(move_list: [Option<T>; SIZE]) -> String
where
    T: std::fmt::Debug,
{
    move_list
        .iter()
        .map_while(|mv| mv.as_ref().map(|m| format!("{:?}", m)))
        .collect::<Vec<String>>()
        .join(", ")
}

fn test_algorithms_once<THandler, TPosition, const DEPTH: usize>(
    searcher: &mut Searcher,
    position_name: &str,
    handler_params: <THandler as GameHandler<TPosition>>::Params,
    startpos_params: <TPosition as GamePosition>::Params,
) where
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
        searcher.reset_leaf_count();
        let s = Instant::now();
        let result: MoveAndPV<THandler, TPosition, DEPTH> = algorithms.N(searcher, &handler, startpos);
        let elapsed = s.elapsed();
        println!(
            "Time elapsed: {} ms, {} us, {} ns",
            elapsed.as_millis().to_string().bright_yellow(),
            elapsed.as_micros().to_string().bright_cyan(),
            elapsed.as_nanos().to_string().bright_blue(),
        );
        println!("Leaf nodes evaluated: {}", searcher.get_leaf_count().to_string().bright_yellow());
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
                "Returned Eval: {}, Recalculated Eval: {}",
                format!("{:?}", result.0).bright_green(),
                format!("{:?}", recalculated_eval).bright_red(),
            );
            println!("Line Given: {}", raw_moves_display(result.1).bright_red());
        }
    });

    searcher.reset_leaf_count();
}

fn test_algorithms_average<THandler, TPosition, const DEPTH: usize>(
    searcher: &mut Searcher,
    position_name: &str,
    times: usize,
    handler_params: Vec<<THandler as GameHandler<TPosition>>::Params>,
    startpos_params: <TPosition as GamePosition>::Params,
    verbose: bool,
)
where
    THandler: GameHandler<TPosition>,
    TPosition: GamePosition,
{
    if handler_params.len() < times {
        panic!("List of GameHandler parameters needs to be equal to or more than the number of iterations");
    }

    let startpos = <TPosition as GamePosition>::startpos(startpos_params);

    let algorithms = (
        root_call_bb,
        root_call_ab,
        root_call_pab,
        root_call_pvs,
        root_call_scout,
        root_call_sss,
    );

    let algorithm_names = [
        "branch_and_bound",
        "alpha_beta",
        "p_alpha_beta",
        "pvs",
        "scout",
        "sss",
    ];

    let mut stats = [
        AlgorithmStats::new(),
        AlgorithmStats::new(),
        AlgorithmStats::new(),
        AlgorithmStats::new(),
        AlgorithmStats::new(),
        AlgorithmStats::new(),
    ];

    if verbose {
        println!(
            "Running {} times: {}",
            times.to_string().bright_cyan(),
            position_name.bright_magenta()
        );
    }

    let mut i: usize = 1;
    for param in handler_params {

        if verbose {
            println!("Iteration {}", i.to_string().bright_cyan());
        }

        let handler = <THandler as GameHandler<TPosition>>::new(param);
        let mut results: [Option<MoveAndPV<THandler, TPosition, DEPTH>>; 6] = [None; 6];

        seq!(N in 0..6 {
            searcher.reset_leaf_count();

            let s = Instant::now();
            let result: MoveAndPV<THandler, TPosition, DEPTH> = algorithms.N(searcher, &handler, startpos);
            let elapsed = s.elapsed();

            let recalculated_eval = eval_from_line(&handler, startpos, result.1);
            if recalculated_eval != result.0 {
                println!(
                    "{}",
                    format!(
                        "INDIVIDUAL MISMATCH (Alg: {}, Returned Eval: {:?}, Recalc Eval: {:?}, Returned Line: {:?})",
                        algorithm_names[N],
                        result.0,
                        recalculated_eval,
                        result.1,
                    )
                        .bright_red(),
                );
            }
            results[N] = Some(result);

            stats[N] += AlgorithmStats {
                avg_leaves: (searcher.get_leaf_count() as f64) / (times as f64),
                avg_ms: (elapsed.as_millis() as f64) / (times as f64),
                avg_us: (elapsed.as_micros() as f64) / (times as f64),
                avg_ns: (elapsed.as_nanos() as f64) / (times as f64),
            };

        });

        let algorithms_match = results
            .iter()
            .skip(1)
            .fold((results[0], true), |(previous_result, all_match), &current_result| {
                (previous_result, all_match && previous_result == current_result)
            })
            .1;

        if !algorithms_match {
            println!("{}", format!("ALGORITHM MISMATCH").bright_red());
            for i in 0..6 {
                println!("Alg: {}, Result: {:?}", algorithm_names[i], results[i]);
            }
        }

        i += 1;
    }

    println!("{}", position_name.bright_magenta());
    for (result, alg_name) in stats.iter().zip(algorithm_names.iter()) {
        let AlgorithmStats { avg_leaves, avg_ms, avg_us, avg_ns } = result;
        println!("Algorithm Tested: {}", alg_name.bright_cyan());
        println!("Average number of leaf nodes evaluated: {}", format!("{:.2}", avg_leaves).bright_yellow());
        println!("Average compute time (milliseconds, 2 d.p.): {} ms", format!("{:.2}", avg_ms).bright_blue());
        println!("Average compute time (microseconds, 2 d.p.): {} us", format!("{:.2}", avg_us).bright_blue());
        println!("Average compute time (nanoseconds, 2 d.p.): {} ns", format!("{:.2}", avg_ns).bright_blue());
    }

    searcher.reset_leaf_count();
}

fn main() {
    let mut searcher = Searcher::new();

    test_algorithms_once::<StockmanHandler, StockmanPos, 4>(
        &mut searcher,
        "Stockman, G.C. (1979)",
        (),
        (),
    );

    test_algorithms_once::<Ut3Handler, Ut3Board, 6>(&mut searcher, "Ultimate Tic-Tac-Toe", (), ());
    test_algorithms_once::<Uniform2bWideHandler, Uniform2bWidePos, 16>(
        &mut searcher,
        "Uniform Tree (Branching Factor = 2)",
        Uniform2bWideParams {
            depth: 16,
            seed: 314159,
        },
        (),
    );

    const DEPTH_WIDTH_PAIRS: [(usize, usize); 24] = [
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
    ];

    seq!(N in 0..24 {
        test_algorithms_once::<UnordIndHypTreeHandler, HypTreePos, { DEPTH_WIDTH_PAIRS[N].0 }>(
            &mut searcher,
            &format!(
                "Unordered-Independent Hypothetical Game Tree (Depth = {}, Width = {})",
                DEPTH_WIDTH_PAIRS[N].0,
                DEPTH_WIDTH_PAIRS[N].1,
            ),
            HypTreeParams {
                depth: DEPTH_WIDTH_PAIRS[N].0,
                width: DEPTH_WIDTH_PAIRS[N].1,
                seed: 314159,
            },
            8,
        );
    });

    seq!(N in 0..24 {
        // Tests all 6 algorithms at once, averaging their results over different seeds
        test_algorithms_average::<UnordIndHypTreeHandler, HypTreePos, { DEPTH_WIDTH_PAIRS[N].0 }>(
            &mut searcher,
            &format!("U({}, {})", DEPTH_WIDTH_PAIRS[N].1, DEPTH_WIDTH_PAIRS[N].0),
            50,
            (314159..314159 + 50)
                .map(|seed| {
                    HypTreeParams {
                        depth: DEPTH_WIDTH_PAIRS[N].0,
                        width: DEPTH_WIDTH_PAIRS[N].1,
                        seed,
                    }
                })
                .collect(),
            DEPTH_WIDTH_PAIRS[N].0,
            false,
        );
    });

    test_algorithms_average::<UnordIndHypTreeHandler, HypTreePos, 8>(
        &mut searcher,
        "U(8, 8)",
        50,
        (314159..314159 + 50)
            .map(|seed| {
                HypTreeParams {
                    depth: 8,
                    width: 8,
                    seed,
                }
            })
            .collect(),
        8,
        true,
    );

}
