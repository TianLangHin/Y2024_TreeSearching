use crate::prelude::*;
use crate::games::stockman::*;
use crate::games::ut3::*;
use crate::search::*;

pub mod prelude;
pub mod games;
pub mod search;

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
    let ut3_eval = p_alpha_beta::<Ut3Handler, Ut3Board>(
        &Ut3Handler::new(),
        Ut3Board::startpos(),
        4,
        4,
    );
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
    let ut3_eval = scout::<Ut3Handler, Ut3Board>(
        &Ut3Handler::new(),
        Ut3Board::startpos(),
        4,
        4,
    );
    println!("Ut3: {:?}", ut3_eval);
    println!("Time elapsed: {} ms", s.elapsed().as_millis());

    // Compute time rises unexpectedly at >= 5ply, and still very slow.
    // Maybe use BTreeMap.
    println!("state space search");
    let s = Instant::now();
    let ut3_eval = sss::<Ut3Handler, Ut3Board>(
        &Ut3Handler::new(),
        Ut3Board::startpos(),
        4,
        4,
    );
    println!("Ut3: {:?}", ut3_eval);
    println!("Time elapsed: {} ms", s.elapsed().as_millis());
}
