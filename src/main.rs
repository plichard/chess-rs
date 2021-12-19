#![feature(iter_zip)]
#![feature(thread_is_running)]
#![feature(step_trait)]
#![feature(destructuring_assignment)]

use std::borrow::BorrowMut;
use crate::board::{Action, Board, Move, MoveNode};
use crate::piece::{Color, Piece, Type};
use rand::Rng;
use std::cmp::{max_by, Ordering};
use std::error::Error;
use std::io;
use std::io::{stdin, stdout, Write};
use std::mem::size_of;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::cell::{Cell, RefCell};
use std::pin::Pin;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::task::{Context, Poll};
use std::thread::{JoinHandle, spawn, Thread};
use sfml::graphics::{Drawable, FloatRect, Shape, Transformable, View};
use sfml::system::{Vector2f, Vector2i};
use sfml::window::Key;
use sfml::window::mouse::Button;
// use sixtyfps::Model;
use crate::utils::{Position};

mod board;
mod piece;
// mod slotvec;
mod utils;
mod piece2;
mod board2;
mod game;

fn run_sfml_gui() {
    use sfml::window::{Style, VideoMode};
    use sfml::graphics::{Sprite, Texture, RenderTarget};
    use std::collections::HashMap;

    let mut board = Board::new_classic_game();

    let mut board_copy = board.clone();

    let (tx_result, rx_result) = sync_channel::<Option<Move>>(1);

    // return;

    let mut settings = sfml::window::ContextSettings::default();
    settings.set_antialiasing_level(4);

    let mut window = sfml::graphics::RenderWindow::new((800, 800), "Chess", Style::DEFAULT, &settings);

    window.set_vertical_sync_enabled(true);

    let mut w_pawn_tex = Texture::from_file("ui/icons/w_pawn.png").unwrap();
    let mut b_pawn_tex = Texture::from_file("ui/icons/b_pawn.png").unwrap();
    let mut w_rook_tex = Texture::from_file("ui/icons/w_rook.png").unwrap();
    let mut b_rook_tex = Texture::from_file("ui/icons/b_rook.png").unwrap();
    let mut w_knight_tex = Texture::from_file("ui/icons/w_knight.png").unwrap();
    let mut b_knight_tex = Texture::from_file("ui/icons/b_knight.png").unwrap();
    let mut w_bishop_tex = Texture::from_file("ui/icons/w_bishop.png").unwrap();
    let mut b_bishop_tex = Texture::from_file("ui/icons/b_bishop.png").unwrap();
    let mut w_queen_tex = Texture::from_file("ui/icons/w_queen.png").unwrap();
    let mut b_queen_tex = Texture::from_file("ui/icons/b_queen.png").unwrap();
    let mut w_king_tex = Texture::from_file("ui/icons/w_king.png").unwrap();
    let mut b_king_tex = Texture::from_file("ui/icons/b_king.png").unwrap();

    let mut white_square = sfml::graphics::RectangleShape::with_size(Vector2f::new(128.0, 128.0));
    white_square.set_fill_color(sfml::graphics::Color::rgb(128, 128, 128));

    let mut black_square = white_square.clone();
    black_square.set_fill_color(sfml::graphics::Color::rgb(50, 50, 50));

    for tex in [&mut w_pawn_tex, &mut b_pawn_tex, &mut w_rook_tex, &mut b_rook_tex,
        &mut w_knight_tex, &mut b_knight_tex, &mut w_bishop_tex, &mut b_bishop_tex, &mut w_queen_tex, &mut b_queen_tex,
        &mut w_king_tex, &mut b_king_tex] {
        tex.set_smooth(true);
    }

    let mut select_square = white_square.clone();
    select_square.set_fill_color(sfml::graphics::Color::rgba(121, 156, 130, 70));
    // select_square.set_outline_color(sfml::graphics::Color::GREEN);
    // select_square.set_outline_thickness(2.0);

    let mut last_move_square = select_square.clone();
    last_move_square.set_fill_color(sfml::graphics::Color::rgba(195, 216, 135, 70));
    // last_move_square.set_outline_color(sfml::graphics::Color::BLUE);
    // last_move_square.set_outline_thickness(2.0);

    let mut legal_move_circle = sfml::graphics::CircleShape::new(128.0 / 4.0, 100);
    legal_move_circle.set_fill_color(sfml::graphics::Color::rgba(16, 97, 38, 128));
    legal_move_circle.set_origin((-32.0, -32.0));

    let mut w_pawn = Sprite::with_texture(&w_pawn_tex);
    w_pawn.set_origin((-(128.0 - w_pawn_tex.size().x as f32) / 2.0, 0.0));
    let mut b_pawn = Sprite::with_texture(&b_pawn_tex);
    b_pawn.set_origin((-(128.0 - b_pawn_tex.size().x as f32) / 2.0, 0.0));
    let mut w_rook = Sprite::with_texture(&w_rook_tex);
    let mut b_rook = Sprite::with_texture(&b_rook_tex);
    let mut w_knight = Sprite::with_texture(&w_knight_tex);
    let mut b_knight = Sprite::with_texture(&b_knight_tex);
    let mut w_bishop = Sprite::with_texture(&w_bishop_tex);
    let mut b_bishop = Sprite::with_texture(&b_bishop_tex);
    let mut w_queen = Sprite::with_texture(&w_queen_tex);
    let mut b_queen = Sprite::with_texture(&b_queen_tex);
    let mut w_king = Sprite::with_texture(&w_king_tex);
    let mut b_king = Sprite::with_texture(&b_king_tex);

    let mut selected: Option<(i32, i32)> = None;

    let font = sfml::graphics::Font::from_file("ui/fonts/Inconsolata-Regular.ttf").unwrap();
    let mut status_text = sfml::graphics::Text::new("Thinking...", &font, 16);

    let mut stop_sender: Option<SyncSender<()>> = None;

    let mut last_move: Option<Move> = None;

    let compute_move = |board: &Board, tx: SyncSender<Option<Move>>| {
        let mut board = board.clone();
        let (tx_stop, rx_stop) = sync_channel(1);

        let thread = std::thread::spawn(move || {
            if let Some(m) = board.find_best_move(6, rx_stop) {
                tx.send(Some(m));
            } else {
                // println!("No valid move found");
                tx.send(None);
            }
        });

        return tx_stop;
    };

    let mut do_compute = false;
    let mut legal_moves = Vec::new();

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            use sfml::window::Event;
            match event {
                Event::Closed => window.close(),
                Event::MouseButtonPressed { button, x, y } => {
                    if button == Button::LEFT {
                        let p = window.map_pixel_to_coords_current_view(Vector2i::new(x, y));
                        let x = p.x as i32 / 128;
                        let y = 7 - p.y as i32 / 128;
                        let next = ((x, y));
                        if let Some(v) = selected {
                            if next != v {
                                if let Some(m) = board.move_from_position(v.0 as i8, v.1 as i8, next.0 as i8, next.1 as i8) {
                                    if legal_moves.contains(&m) {
                                        last_move = Some(m);
                                        board.push_move(m);
                                        selected = None;
                                        legal_moves.clear();
                                    } else {
                                        selected = None;
                                        legal_moves.clear();
                                    }
                                } else {
                                    if let Some(piece) = board.piece_at(&Position::new(next.0 as i8, next.1 as i8)) {
                                        if piece.color == board.current_color() {
                                            selected = Some(next);
                                            legal_moves = board.collect_piece_moves(piece).into_iter().map(|m| m.m).collect();
                                        }
                                    } else {
                                        selected = None;
                                        legal_moves.clear();
                                    }
                                }
                            }
                        } else {
                            if let Some(piece) = board.piece_at(&Position::new(next.0 as i8, next.1 as i8)) {
                                if piece.color == board.current_color() {
                                    selected = Some(next);
                                    legal_moves = board.collect_piece_moves(piece).into_iter().map(|m| m.m).collect();
                                }
                            } else {
                                selected = None;
                                legal_moves.clear();
                            }
                        }
                    }


                    if button == Button::RIGHT {
                        selected = None;
                        legal_moves.clear();
                    }
                }
                Event::KeyPressed { code, alt, ctrl, shift, system } => {
                    if code == Key::LEFT {
                        board.pop_move();
                    } else if code == Key::SPACE {
                        do_compute = true;
                    }
                }
                _ => {}
            }
        }

        let mut computing = false;

        if let Some(sender) = &stop_sender {
            if let Ok(m) = rx_result.try_recv() {
                if let Some(m) = m {
                    board.push_move(m);
                    last_move = Some(m);
                    stop_sender = None;
                    // stop_sender = Some(compute_move(&board, tx_result.clone()));
                }
            } else {
                computing = true;
            }
        } else if do_compute {
            stop_sender = Some(compute_move(&board, tx_result.clone()));
            do_compute = false;
        }

        window.clear(sfml::graphics::Color::rgb(0, 0, 0));
        window.set_view(&View::from_rect(&FloatRect::new(0.0, 0.0, 128.0 * 8.0, 128.0 * 8.0)));

        for y in 0..8 {
            for x in 0..8 {
                let px = x as f32 * 128.0;
                let py = (7 - y) as f32 * 128.0;
                if (x + y) % 2 == 0 {
                    black_square.set_position((px, py));
                    window.draw(&black_square);
                } else {
                    white_square.set_position((px, py));
                    window.draw(&white_square);
                }
            }
        }

        if let Some(m) = last_move {
            let pos = match m.action {
                Action::Evaluation { .. } => None,
                Action::Move { from, to } => {
                    Some((from.position.x, from.position.y, to.position.x, to.position.y))
                }
                Action::Capture { piece, target } => {
                    Some((piece.position.x, piece.position.y, target.position.x, target.position.y))
                }
                Action::Promote { old_piece, new_piece } => {
                    Some((old_piece.position.x, old_piece.position.y, new_piece.position.x, new_piece.position.y))
                }
            };

            if let Some((x1, y1, x2, y2)) = pos {
                let px1 = x1 as f32 * 128.0;
                let py1 = (7 - y1) as f32 * 128.0;

                let px2 = x2 as f32 * 128.0;
                let py2 = (7 - y2) as f32 * 128.0;

                last_move_square.set_position((px1, py1));
                window.draw(&last_move_square);

                last_move_square.set_position((px2, py2));
                window.draw(&last_move_square);
            }
        }

        if let Some((x, y)) = selected {
            let px = 128.0 * x as f32;
            let py = 128.0 * (7 - y) as f32;
            select_square.set_position((px, py));
            window.draw(&select_square);
        }

        let state = board.state();
        for y in 0..8 {
            for x in 0..8 {
                let px = x as f32 * 128.0;
                let py = (7 - y) as f32 * 128.0;
                if let Some(piece) = &state[x][y].piece {
                    use Color::*;
                    use Type::*;
                    let sprite = match (piece.color, piece.t) {
                        (White, Pawn) => &mut w_pawn,
                        (Black, Pawn) => &mut b_pawn,
                        (White, Rook) => &mut w_rook,
                        (Black, Rook) => &mut b_rook,
                        (White, Knight) => &mut w_knight,
                        (Black, Knight) => &mut b_knight,
                        (White, Bishop) => &mut w_bishop,
                        (Black, Bishop) => &mut b_bishop,
                        (White, Queen) => &mut w_queen,
                        (Black, Queen) => &mut b_queen,
                        (White, King) => &mut w_king,
                        (Black, King) => &mut b_king,
                    };

                    sprite.set_position((px, py));
                    window.draw(sprite);
                }
            }
        }

        for m in &legal_moves {
            let (x, y) = match m.action {
                Action::Evaluation { .. } => { unreachable!() }
                Action::Move { to, .. } => (to.position.x, to.position.y),
                Action::Capture { target, .. } => (target.position.x, target.position.y),
                Action::Promote { new_piece, .. } => (new_piece.position.x, new_piece.position.y)
            };
            let px = x as f32 * 128.0;
            let py = (7 - y) as f32 * 128.0;

            legal_move_circle.set_position((px, py));
            window.draw(&legal_move_circle);
        }


        if computing {
            status_text.set_string("Thinking...");
            status_text.set_character_size(42);
            status_text.set_fill_color(sfml::graphics::Color::rgb(50, 255, 50));
            window.draw(&status_text);
        }

        window.display();
    }
}


fn main() {
    // use piece2::{Piece, Color, Type, Position};
    // use game::Game;
    //
    // let mut game = Game::new_pawn_only();
    // game.do_testing();

    // run_sfml_gui();
    return;
}
