#![feature(iter_zip)]

use crate::board::{Board, Move, MoveNode, Action};
use crate::piece::Color;
use rand::Rng;
use std::cmp::Ordering;
use std::error::Error;
use std::io;
use std::io::{stdin, stdout, Write};
use std::mem::size_of;

mod board;
mod piece;
mod utils;


fn main() {

    println!("sizeof Move: {}", std::mem::size_of::<board::Move>());

    let mut board = Board::new_classic_game();

    let mut rng = rand::thread_rng();

    // draw_board(&mut terminal, board.clone());

    // std::thread::sleep_ms(1000);
    let mut buffer = String::new();
    // let mut i = 0;
    loop {
        
        // print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        board.print();
        // let moves = board.collect_all_moves(board.current_color(), false);
        // if moves.is_empty() {
        //     break;
        // }
        // let m = moves[0];
        if board.current_color() == Color::Black {
            let mut root_node : MoveNode = Move::evaluate(0).into();
            let m = board.search(6, Move::evaluate(-i32::MAX).into(), Move::evaluate(i32::MAX).into(), &mut root_node, false);
            if !m.is_valid() {
                break;
            }
            // println!("{:?}", root_node);
            println!("Evaluation ({}): {}", root_node.recursive_children_count(), m.value());
            board.push_move(m);
        } else {
            buffer.clear();
            print!("Your move:");
            stdout().flush();
            stdin().read_line(&mut buffer);
            while !board.parse_move(&buffer) {
                buffer.clear();
                println!("Wrong, try again");
                print!("Your move:");
                stdout().flush();
                stdin().read_line(&mut buffer);
            }
        }
    }
    // print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    board.print();

    // println!("Executed {} moves", i);

    // while board.depth() > 0 {
    //     board.pop_move();
    //     board.print();
    //     std::thread::sleep_ms(50);
    // }


    // board.print();
}
