#![feature(iter_zip)]


use std::borrow::BorrowMut;
use crate::board::{Action, Board, Move, MoveNode};
use crate::piece::{Color, Type};
use rand::Rng;
use std::cmp::Ordering;
use std::error::Error;
use std::io;
use std::io::{stdin, stdout, Write};
use std::mem::size_of;
use std::path::Path;
use std::sync::Arc;
use std::cell::{Cell, RefCell};
use sixtyfps::Model;

mod board;
mod piece;
// mod slotvec;
mod utils;

sixtyfps::include_modules!();


struct ChessWrapper {
    p1: Cell<Option<(i32, i32)>>,
    p2: Cell<Option<(i32, i32)>>,

    images: std::collections::HashMap<(Color, Type), sixtyfps::Image>,
    model: std::rc::Rc<sixtyfps::VecModel<CellData>>,
    board: RefCell<Board>,
}

impl ChessWrapper {
    fn new(win: &MainWindow) -> Self {

        let mut images = std::collections::HashMap::new();

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

        images.insert((Color::White, Type::Pawn), w_pawn);
        images.insert((Color::Black, Type::Pawn), b_pawn);

        images.insert((Color::White, Type::Rook), w_rook);
        images.insert((Color::Black, Type::Rook), b_rook);

        images.insert((Color::White, Type::Knight), w_knight);
        images.insert((Color::Black, Type::Knight), b_knight);

        images.insert((Color::White, Type::Bishop), w_bishop);
        images.insert((Color::Black, Type::Bishop), b_bishop);

        images.insert((Color::White, Type::Queen), w_queen);
        images.insert((Color::Black, Type::Queen), b_queen);

        images.insert((Color::White, Type::King), w_king);
        images.insert((Color::Black, Type::King), b_king);

        let mut cells : Vec<CellData> = Vec::new();
        for i in 0..64 {
            cells.push(CellData{selected: false, img: Default::default()});
        }

        let cells_model = std::rc::Rc::new(sixtyfps::VecModel::from(cells));
        win.set_cells(sixtyfps::ModelHandle::new(cells_model.clone()));

        Self {
            p1: Cell::new(None),
            p2: Cell::new(None),

            board: RefCell::new(Board::new_classic_game()),
            model: cells_model,
            images
        }
    }
    fn on_click(&self, win: &MainWindow, index: usize, x: i32, y: i32) {
        match (self.p1.get(), self.p2.get()) {
            (None, None) => {
                println!("Selected first");
                self.p1.set(Some((x, y)));
                self.set_selection(win, index, true);
            }
            (Some(p1), None) => {
                println!("Selected second");
                self.p2.set(Some((x, y)));
                self.set_selection(win, index, true);
            }
            (Some(p1), Some(p2)) => {
                println!("Already selected all");
                self.clear_selection(win);
                self.p1.set(None);
                self.p2.set(None);
            },
            (_, _) => {}
        }
    }

    fn clear_selection(&self, win: &MainWindow) {
        let model = win.get_cells();
        for i in 0..model.row_count() {
            let mut data = model.row_data(i);
            data.selected = false;
            model.set_row_data(i, data);
        }
    }

    fn set_selection(&self, win: &MainWindow, index: usize, value: bool) {
        let model = win.get_cells();
        let mut data = model.row_data(index);
        data.selected = value;
        model.set_row_data(index, data);
    }

    fn set_piece(&self, win: &MainWindow, x: i32, y: i32, t: Type) {

    }
}

fn test_sixty() {
    use sixtyfps::Model;
    let main_window = MainWindow::new();



    let main_weak = main_window.as_weak();

    // let test = std::rc::Rc::new(ChessWrapper::new());
    //
    // let test_copy = test.clone();

    let test_copy = ChessWrapper::new(&main_window);

    main_window.on_clicked(move |id, x, y| {
        let id = id as usize;
        if let Some(win) = main_weak.upgrade() {
            test_copy.on_click(&win, id, x, y);
            // let cells = win.get_cells();
            // cells.row_data(4);
            // let mut c = cells.row_data(id);
            // c.img = w_queen.clone();
            //
            // // println!("{:?}", c);
            //
            // cells.set_row_data(id, c);

            //  [id as usize].img = w_queen.clone();
            // cells[id as usize].img = w_queen.clone();
        }
    });

    main_window.run();
}

fn main() {
    test_sixty();
    return;
    println!("sizeof Move: {}", std::mem::size_of::<board::Move>());

    let mut board = Board::new_classic_game();
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
        // board.compute_attacked_cells();
        // board.print_attacked_cells();
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
