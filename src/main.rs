use crate::board::{Board, Move};
use crate::piece::Color;
use rand::Rng;
use std::cmp::Ordering;
use std::error::Error;
use std::io;
use std::mem::size_of;

mod board;
mod piece;
mod utils;


fn main() {

    let mut board = Board::new_pawn_game();

    let mut rng = rand::thread_rng();



    // draw_board(&mut terminal, board.clone());

    std::thread::sleep_ms(1000);


    // let mut i = 0;
    // board.print();
    for _ in 0..100 {
        loop {
            let mut moves = board.collect_all_moves(board.current_color());
            if moves.is_empty() {
                break;
            }
            // board.sort_moves(&mut moves);
            board.push_move(moves[rng.gen_range(0..moves.len())]);
            // i += 1;
            // board.print();
            // std::thread::sleep_ms(50);
        }

        // println!("Executed {} moves", i);

        while board.depth() > 0 {
            board.pop_move();
            // board.print();
            // std::thread::sleep_ms(50);
        }
    }

    board.print();
}
