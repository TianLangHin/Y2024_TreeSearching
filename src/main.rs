use crate::games::chess::*;
use crate::games::stockman::*;
use crate::games::ut3::*;
use crate::prelude::*;
use crate::search::*;

pub mod games;
pub mod prelude;
pub mod search;

use rayon::prelude::*;

use std::time::Instant;

fn test_stockman_tree() {
    let handler = StockmanHandler::new();
    let mut nodes = vec![StockmanPos::startpos()];
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

fn square_to_string(sq: u64) -> String {
    let f = ["a", "b", "c", "d", "e", "f", "g", "h"];
    let r = ["1", "2", "3", "4", "5", "6", "7", "8"];
    format!("{}{}", f[(sq & 7) as usize], r[((sq >> 3) & 7) as usize])
}

fn move_string(mv: u64, side: u64) -> String {
    let o = if side == 1 { flip_square(mv & 0x3f) } else { mv & 0x3f };
    let d = if side == 1 { flip_square((mv >> 6) & 0x3f) } else { (mv >> 6) & 0x3f };
    let f = (mv >> 12) & 0x3;
    let p = (mv >> 14) & 0x3;

    match f {
        0 => format!("{}{}", square_to_string(o), square_to_string(d)),
        1 => {
            format!(
                "{}{}{}",
                square_to_string(o),
                square_to_string(d),
                if p == 0 {
                    "q"
                } else if p == 1 {
                    "r"
                } else if p == 2 {
                    "b"
                } else if p == 3 {
                    "n"
                } else {
                    panic!("promote flag invalid")
                }
            )
        }
        2 => (if d == 2 { "e1c1" } else { "e1g1" }).to_string(),
        3 => format!("{}{}ep", square_to_string(o), square_to_string(d)),
        _ => panic!("move flag invalid: {}", f),
    }
}

fn perft(depth: usize, pos: ChessPos, handler: &ChessHandler) -> u64 {
    if depth == 1 {
        return handler.get_legal_moves(pos).count() as u64;
    }
    handler.get_legal_moves(pos)
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
    let sum: u64 = handler.get_legal_moves(pos)
        .map(|mv| {
            let num = perft(depth - 1, pos.play_move(mv), handler);
            println!("{}: {num}", move_string(mv, (pos.squares >> 19) & 1));
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
    let sum: u64 = handler.get_legal_moves(pos)
        .collect::<Vec<_>>()
        .par_iter()
        .map(|&mv| {
            let num = perft(depth - 1, pos.play_move(mv), handler);
            println!("{}: {num}", move_string(mv, (pos.squares >> 19) & 1));
            num
        })
        .sum();
    println!("Nodes searched: {sum}");
    println!("Time elapsed: {} ms", s.elapsed().as_millis());
}

fn main() {
    test_stockman_tree();

    println!("Stockman");

    println!("branch_and_bound");
    let s = Instant::now();
    let stockman_eval = branch_and_bound::<StockmanHandler, StockmanPos>(
        &StockmanHandler::new(),
        StockmanPos::startpos(),
        4,
        4,
        StockmanHandler::EVAL_MAXIMUM,
    );
    println!("Stockman: {:?}", stockman_eval);
    println!("Time elapsed: {} ms", s.elapsed().as_millis());

    println!("alpha_beta");
    let s = Instant::now();
    let stockman_eval = alpha_beta::<StockmanHandler, StockmanPos>(
        &StockmanHandler::new(),
        StockmanPos::startpos(),
        4,
        4,
        StockmanHandler::EVAL_MINIMUM,
        StockmanHandler::EVAL_MAXIMUM,
    );
    println!("Stockman: {:?}", stockman_eval);
    println!("Time elapsed: {} ms", s.elapsed().as_millis());

    println!("p_alpha_beta");
    let s = Instant::now();
    let stockman_eval = p_alpha_beta::<StockmanHandler, StockmanPos>(
        &StockmanHandler::new(),
        StockmanPos::startpos(),
        4,
        4,
    );
    println!("Stockman: {:?}", stockman_eval);
    println!("Time elapsed: {} ms", s.elapsed().as_millis());

    println!("pvs");
    let s = Instant::now();
    let stockman_eval = pvs::<StockmanHandler, StockmanPos>(
        &StockmanHandler::new(),
        StockmanPos::startpos(),
        4,
        4,
        StockmanHandler::EVAL_MINIMUM,
        StockmanHandler::EVAL_MAXIMUM,
    );
    println!("Stockman: {:?}", stockman_eval);
    println!("Time elapsed: {} ms", s.elapsed().as_millis());

    println!("scout");
    let s = Instant::now();
    let stockman_eval = scout::<StockmanHandler, StockmanPos>(
        &StockmanHandler::new(),
        StockmanPos::startpos(),
        10,
        10,
    );
    println!("Stockman: {:?}", stockman_eval);
    println!("Time elapsed: {} ms", s.elapsed().as_millis());

    println!("state space search");
    let s = Instant::now();
    let stockman_eval = sss::<StockmanHandler, StockmanPos>(
        &StockmanHandler::new(),
        StockmanPos::startpos(),
        10,
        10,
    );
    println!("Stockman: {:?}", stockman_eval);
    println!("Time elapsed: {} ms", s.elapsed().as_millis());

    println!("Ut3");

    println!("branch_and_bound");
    let s = Instant::now();
    let ut3_eval = branch_and_bound::<Ut3Handler, Ut3Board>(
        &Ut3Handler::new(),
        Ut3Board::startpos(),
        4,
        4,
        Ut3Handler::EVAL_MAXIMUM,
    );
    println!("Ut3: {:?}", ut3_eval);
    println!("Time elapsed: {} ms", s.elapsed().as_millis());

    println!("alpha_beta");
    let s = Instant::now();
    let ut3_eval = alpha_beta::<Ut3Handler, Ut3Board>(
        &Ut3Handler::new(),
        Ut3Board::startpos(),
        4,
        4,
        Ut3Handler::EVAL_MINIMUM,
        Ut3Handler::EVAL_MAXIMUM,
    );
    println!("Ut3: {:?}", ut3_eval);
    println!("Time elapsed: {} ms", s.elapsed().as_millis());

    println!("p_alpha_beta");
    let s = Instant::now();
    let ut3_eval =
        p_alpha_beta::<Ut3Handler, Ut3Board>(&Ut3Handler::new(), Ut3Board::startpos(), 4, 4);
    println!("Ut3: {:?}", ut3_eval);
    println!("Time elapsed: {} ms", s.elapsed().as_millis());

    println!("pvs");
    let s = Instant::now();
    let ut3_eval = pvs::<Ut3Handler, Ut3Board>(
        &Ut3Handler::new(),
        Ut3Board::startpos(),
        4,
        4,
        Ut3Handler::EVAL_MINIMUM,
        Ut3Handler::EVAL_MAXIMUM,
    );
    println!("Ut3: {:?}", ut3_eval);
    println!("Time elapsed: {} ms", s.elapsed().as_millis());

    println!("scout");
    let s = Instant::now();
    let ut3_eval = scout::<Ut3Handler, Ut3Board>(&Ut3Handler::new(), Ut3Board::startpos(), 4, 4);
    println!("Ut3: {:?}", ut3_eval);
    println!("Time elapsed: {} ms", s.elapsed().as_millis());

    // Compute time rises unexpectedly at >= 5ply, and still very slow.
    // Maybe use BTreeMap.
    println!("state space search");
    let s = Instant::now();
    let ut3_eval = sss::<Ut3Handler, Ut3Board>(&Ut3Handler::new(), Ut3Board::startpos(), 4, 4);
    println!("Ut3: {:?}", ut3_eval);
    println!("Time elapsed: {} ms", s.elapsed().as_millis());

    let pos = ChessPos::startpos();
    println!("perft 6");
    perft_div_main_par(6, pos, &ChessHandler::new());
    perft_div_main(6, pos, &ChessHandler::new());
}
