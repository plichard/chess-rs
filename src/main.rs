use crate::board::{Board, Move, Action};
use crate::piece::Color;
use rand::Rng;
use std::cmp::Ordering;
use std::error::Error;
use std::io;
use std::io::stdin;
use std::mem::size_of;

mod board;
mod piece;
mod utils;


fn main() {

    let mut board = Board::new_classic_game();

    let mut rng = rand::thread_rng();

    // draw_board(&mut terminal, board.clone());

    // std::thread::sleep_ms(1000);
    let mut buffer = String::new();
    // let mut i = 0;
    loop {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        board.print();
        // let moves = board.collect_all_moves(board.current_color(), false);
        // if moves.is_empty() {
        //     break;
        // }
        // let m = moves[0];
        let m = board.search(7, Move::evaluate(-i64::MAX), Move::evaluate(i64::MAX), false);
        if !m.is_valid() {
            break;
        }
        println!("Evaluation: {}", m.evaluation());
        // stdin().read_line(&mut buffer);
        board.push_move(m);
    }
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    board.print();

    // println!("Executed {} moves", i);

    // while board.depth() > 0 {
    //     board.pop_move();
    //     board.print();
    //     std::thread::sleep_ms(50);
    // }


    // board.print();
}
