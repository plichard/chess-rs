#![feature(iter_zip)]

use crate::board::{Action, Board, Move, MoveNode};
use crate::piece::{Color, Type};
use rand::Rng;
use std::cmp::Ordering;
use std::error::Error;
use std::io;
use std::io::{stdin, stdout, Write};
use std::mem::size_of;
use std::path::Path;

mod board;
mod piece;
mod slotvec;
mod utils;

sixtyfps::include_modules!();

fn test_sixty() {
    use sixtyfps::Model;
    let main_window = MainWindow::new();

    let w_pawn = sixtyfps::Image::load_from_path(Path::new("ui/icons/w_pawn.png")).unwrap();
    let b_pawn = sixtyfps::Image::load_from_path(Path::new("ui/icons/b_pawn.png")).unwrap();

    let w_rook = sixtyfps::Image::load_from_path(Path::new("ui/icons/w_rook.png")).unwrap();
    let b_rook = sixtyfps::Image::load_from_path(Path::new("ui/icons/b_rook.png")).unwrap();

    let w_knight = sixtyfps::Image::load_from_path(Path::new("ui/icons/w_knight.png")).unwrap();
    let b_knight = sixtyfps::Image::load_from_path(Path::new("ui/icons/b_knight.png")).unwrap();

    let w_bishop = sixtyfps::Image::load_from_path(Path::new("ui/icons/w_bishop.png")).unwrap();
    let b_bishop = sixtyfps::Image::load_from_path(Path::new("ui/icons/b_bishop.png")).unwrap();

    let w_king = sixtyfps::Image::load_from_path(Path::new("ui/icons/w_king.png")).unwrap();
    let b_king = sixtyfps::Image::load_from_path(Path::new("ui/icons/b_king.png")).unwrap();

    let w_queen = sixtyfps::Image::load_from_path(Path::new("ui/icons/w_queen.png")).unwrap();
    let b_queen = sixtyfps::Image::load_from_path(Path::new("ui/icons/b_queen.png")).unwrap();

    let mut cells: Vec<CellData> = main_window.get_cells().iter().collect();
    cells.resize(
        64,
        CellData {
            id: 0
        },
    );

    for x in 0..8 {
        cells[x + 8 * 1].id = 1;

        cells[x + 8 * 6].id = 1;
    }

    let cells_model = std::rc::Rc::new(sixtyfps::VecModel::from(cells));
    main_window.set_cells(sixtyfps::ModelHandle::new(cells_model));

    let main_weak = main_window.as_weak();
    main_window.on_clicked(move || {
        println!("Clicked!");
    });
    main_window.run();
}

fn main() {
    test_sixty();
    return;
    println!("sizeof Move: {}", std::mem::size_of::<board::Move>());

    let mut board = Board::new_empty_game();
    // board.add_new_piece(Color::White, Type::Pawn, 0, 0);
    // board.add_new_piece(Color::Black, Type::Knight, 7, 7);
    board.compute_attacked_cells();

    let mut rng = rand::thread_rng();

    // draw_board(&mut terminal, board.clone());

    // std::thread::sleep_ms(1000);
    let mut buffer = String::new();
    // let mut i = 0;
    loop {
        // print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        board.print();
        board.compute_attacked_cells();
        board.print_attacked_cells();
        // let moves = board.collect_all_moves(board.current_color(), false);
        // if moves.is_empty() {
        //     break;
        // }
        // let m = moves[0];
        if board.current_color() == Color::Black {
            let mut root_node: MoveNode = Move::evaluate(0).into();
            let m = board.search(
                5,
                Move::evaluate(-i32::MAX).into(),
                Move::evaluate(i32::MAX).into(),
                &mut root_node,
                false,
            );
            if !m.is_valid() {
                break;
            }
            // println!("{:?}", root_node);
            println!(
                "Evaluation ({}): {}",
                root_node.recursive_children_count(),
                m.value()
            );
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
