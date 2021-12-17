use std::iter::zip;

use crate::piece::{Color, Piece, PieceIndex, Type};
// use crate::slotvec::StaticSlotVec;
use crate::utils::Position;
use rand::rngs::ThreadRng;
use std::cmp::Ordering;
use std::io::Write;
use std::ops::{Neg, Shl};
use std::process::Output;
use termcolor::{ColorChoice, ColorSpec, WriteColor};
use std::sync::mpsc::{SyncSender, Receiver};

#[derive(Copy, Clone, Debug)]
pub struct Move {
    pub score: i32,
    pub action: Action,
}

impl Into<MoveNode> for Move {
    fn into(self) -> MoveNode {
        MoveNode {
            m: self,
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MoveNode {
    pub m: Move,
    children: Vec<MoveNode>,
}

impl MoveNode {
    pub fn recursive_children_count(&self) -> usize {
        self.children.len()
            + self
            .children
            .iter()
            .map(|c| c.recursive_children_count())
            .sum::<usize>()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Action {
    Evaluation { score: i32 },
    Move { from: Piece, to: Piece },
    Capture { piece: Piece, target: Piece },
    Promote { old_piece: Piece, new_piece: Piece },
}

impl Neg for Move {
    type Output = Move;

    fn neg(self) -> Self::Output {
        Self::Output {
            score: -self.score,
            action: self.action,
        }
    }
}

impl Move {
    pub fn value(&self) -> i32 {
        match self.action {
            Action::Move { .. } => 0,
            Action::Capture { piece, target } => target.value() * 10 - piece.value(),
            Action::Evaluation { score } => score,
            Action::Promote { new_piece, .. } => new_piece.value() * 10,
        }
    }

    pub fn evaluate(score: i32) -> Self {
        Self {
            score,
            action: Action::Evaluation { score },
        }
    }

    pub fn capture_piece(piece: Piece, target: Piece) -> Self {
        Self {
            score: 0,
            action: Action::Capture { piece, target },
        }
    }

    pub fn move_piece(piece: Piece, to: Position) -> Self {
        Self {
            score: 0,
            action: Action::Move {
                from: piece,
                to: piece.moved(to),
            },
        }
    }

    pub fn promote(piece: Piece, to: Position, t: Type) -> Self {
        Self {
            score: 0,
            action: Action::Promote {
                old_piece: piece,
                new_piece: Piece {
                    t,
                    position: to,
                    index: piece.index,
                    color: piece.color,
                },
            },
        }
    }

    // pub fn promote_piece(from: Piece, to: Position, t: Type) -> Self {
    //     let piece = Piece {
    //         t,
    //         position: to,
    //         color: from.color,
    //         index: from.index
    //     };
    //     Self {
    //         evaluation: 0,
    //         action: Action::Promote {from, to: piece}
    //     }
    // }

    pub fn is_valid(&self) -> bool {
        match self.action {
            _ => true,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Cell {
    pub piece: Option<Piece>,
    pub attacking_white_pieces: i8,
    pub attacking_black_pieces: i8,
}

impl Cell {
    pub fn empty() -> Self {
        Self {
            piece: None,
            attacking_white_pieces: 0,
            attacking_black_pieces: 0,
        }
    }
}

#[derive(Clone)]
pub struct Board {
    white_pieces: [Option<Piece>; 16],
    black_pieces: [Option<Piece>; 16],

    used_white_pieces: usize,
    used_black_pieces: usize,

    cells: [[Cell; 8]; 8],
    move_stack: Vec<Move>,

    green: ColorSpec,
    red: ColorSpec,

    white_piece_count: i8,
    black_piece_count: i8,

    move_count: i64,
}

impl Board {
    pub fn new_empty_game() -> Self {
        let mut green = ColorSpec::new();
        green.set_fg(Some(termcolor::Color::Green));

        let mut red = ColorSpec::new();
        red.set_fg(Some(termcolor::Color::Red));

        Self {
            white_pieces: [None; 16],
            black_pieces: [None; 16],
            cells: [[Cell::empty(); 8]; 8],
            move_stack: Vec::new(),
            green,
            red,
            used_black_pieces: 0,
            used_white_pieces: 0,

            white_piece_count: 0,
            black_piece_count: 0,

            move_count: 0,
        }
    }

    pub fn state(&self) -> &[[Cell; 8]; 8] {
        &self.cells
    }

    pub fn move_from_position(&mut self, x1: i8, y1: i8, x2: i8, y2: i8) -> Option<Move> {
        if x1 < 0 || x1 > 7 {
            return None;
        }

        if y1 < 0 || y1 > 7 {
            return None;
        }

        if x2 < 0 || x2 > 7 {
            return None;
        }

        if y2 < 0 || y2 > 7 {
            return None;
        }

        let p1 = Position::new(x1, y1);
        let p2 = Position::new(x2, y2);

        if let Some(piece) = self.piece_at(&p1) {
            if piece.color == self.current_color() {} else {
                println!("This is not your piece");
            }
        } else {
            println!("There is no piece there");
            return None;
        }

        let piece = self.piece_at(&p1).unwrap();

        return if let Some(target) = self.piece_at(&p2) {
            if target.color != self.current_color() {
                let m = Move::capture_piece(piece, *target);
                Some(m)
            } else {
                println!("Cannot capture your own piece");
                None
            }
        } else {
            if piece.t == Type::Pawn {
                if piece.color == Color::White && p2.y == 7 {
                    Some(Move::promote(piece, p2, Type::Queen))
                } else if piece.color == Color::Black && p2.y == 0 {
                    Some(Move::promote(piece, p2, Type::Queen))
                } else {
                    Some(Move::move_piece(piece, p2))
                }
            } else {
                Some(Move::move_piece(piece, p2))
            }
        };
    }

    pub fn parse_move(&mut self, msg: &String) -> bool {
        let msg = msg.as_bytes();
        if msg.len() < 4 {
            return false;
        }
        let char_to_n = |c: u8| c as i8 - 'a' as i8;
        let digit_to_n = |c: u8| c as i8 - '1' as i8;
        let x1 = char_to_n(msg[0]);
        let y1 = digit_to_n(msg[1]);
        let x2 = char_to_n(msg[2]);
        let y2 = digit_to_n(msg[3]);

        self.move_from_position(x1, y1, x2, y2).is_some()
    }

    pub fn new_promote_game() -> Self {
        let mut game = Board::new_empty_game();

        game.add_new_piece(Color::White, Type::Pawn, 0, 1);
        game.add_new_piece(Color::Black, Type::Pawn, 7, 6);

        game
    }

    pub fn new_two_pawn_game() -> Self {
        let mut game = Board::new_empty_game();
        game.add_new_piece(Color::White, Type::Pawn, 0, 1);
        game.add_new_piece(Color::White, Type::Pawn, 5, 1);

        game.add_new_piece(Color::Black, Type::Pawn, 1, 6);
        game.add_new_piece(Color::Black, Type::Pawn, 5, 6);

        game
    }

    pub fn new_classic_game() -> Self {
        let mut game = Board::new_pawn_game();

        game.add_new_piece(Color::White, Type::Rook, 0, 0);
        game.add_new_piece(Color::White, Type::Rook, 7, 0);

        game.add_new_piece(Color::Black, Type::Rook, 0, 7);
        game.add_new_piece(Color::Black, Type::Rook, 7, 7);

        game.add_new_piece(Color::White, Type::King, 4, 0);
        game.add_new_piece(Color::Black, Type::King, 4, 7);

        game.add_new_piece(Color::White, Type::Queen, 3, 0);
        game.add_new_piece(Color::Black, Type::Queen, 3, 7);

        game.add_new_piece(Color::White, Type::Bishop, 2, 0);
        game.add_new_piece(Color::White, Type::Bishop, 5, 0);

        game.add_new_piece(Color::Black, Type::Bishop, 2, 7);
        game.add_new_piece(Color::Black, Type::Bishop, 5, 7);

        game.add_new_piece(Color::White, Type::Knight, 1, 0);
        game.add_new_piece(Color::White, Type::Knight, 6, 0);

        game.add_new_piece(Color::Black, Type::Knight, 1, 7);
        game.add_new_piece(Color::Black, Type::Knight, 6, 7);

        game
    }

    pub fn new_win_game() -> Self {
        let mut game = Board::new_empty_game();
        game.add_new_piece(Color::White, Type::Rook, 0, 0);
        game.add_new_piece(Color::White, Type::Rook, 1, 0);
        game.add_new_piece(Color::White, Type::Rook, 2, 0);

        game.add_new_piece(Color::Black, Type::Rook, 5, 0);
        game.add_new_piece(Color::Black, Type::Rook, 5, 1);
        game.add_new_piece(Color::Black, Type::Rook, 5, 2);

        game.add_new_piece(Color::White, Type::King, 0, 7);
        game.add_new_piece(Color::Black, Type::King, 7, 5);

        game
    }

    pub fn new_test_game() -> Self {
        let mut game = Board::new_pawn_game();

        game.add_new_piece(Color::White, Type::Rook, 0, 0);
        game.add_new_piece(Color::White, Type::Rook, 2, 0);
        game.add_new_piece(Color::White, Type::Rook, 4, 0);
        game.add_new_piece(Color::White, Type::Rook, 7, 0);

        game.add_new_piece(Color::Black, Type::Rook, 0, 7);
        game.add_new_piece(Color::Black, Type::Rook, 2, 7);
        game.add_new_piece(Color::Black, Type::Rook, 4, 7);
        game.add_new_piece(Color::Black, Type::Rook, 7, 7);

        game
    }

    pub fn new_pawn_game() -> Self {
        let mut game = Board::new_empty_game();

        for i in 0..8 {
            game.add_new_piece(Color::White, Type::Pawn, i, 1);
            game.add_new_piece(Color::Black, Type::Pawn, i, 6);
        }

        game
    }

    pub fn new_single_pawn_game() -> Self {
        let mut game = Board::new_empty_game();
        game.add_new_piece(Color::White, Type::Pawn, 4, 4);
        game.add_new_piece(Color::Black, Type::Pawn, 5, 5);
        game.add_new_piece(Color::Black, Type::Pawn, 7, 5);

        game
    }

    pub fn add_new_piece(&mut self, color: Color, t: Type, x: i8, y: i8) {
        if color == Color::White {
            self.white_pieces[self.used_white_pieces] = Some(Piece {
                t,
                position: Position { x, y },
                color,
                index: self.used_white_pieces as u8,
            });
            self.cells[x as usize][y as usize].piece = self.white_pieces[self.used_white_pieces];
            self.used_white_pieces += 1;
        } else {
            self.black_pieces[self.used_black_pieces] = Some(Piece {
                t,
                position: Position { x, y },
                color,
                index: self.used_black_pieces as u8,
            });
            self.cells[x as usize][y as usize].piece = self.black_pieces[self.used_black_pieces];
            self.used_black_pieces += 1;
        }
    }

    pub fn find_best_move(&mut self, depth: i32, rx: Receiver<()>) -> Option<Move> {
        self.move_count = 0;
        let mut root_node: MoveNode = Move::evaluate(0).into();
        let m = self.search(
            depth,
            Move::evaluate(-i32::MAX).into(),
            Move::evaluate(i32::MAX).into(),
            &mut root_node,
            false,
            &rx,
        );
        if !m.is_valid() {
            return None;
        }

        println!("score = {}, move count = {}", m.score, self.move_count);

        return Some(m);
    }

    pub fn search(
        &mut self,
        depth: i32,
        mut alpha: Move,
        beta: Move,
        parent: &mut MoveNode,
        only_captures: bool,
        rx: &Receiver<()>,
    ) -> Move {
        if rx.try_recv().is_ok() {
            return Move::evaluate(self.evaluate_position());
        }

        if let Action::Capture { target, .. } = &parent.m.action {
            if target.t == Type::King {
                return Move::evaluate(self.evaluate_position());
            }
        }

        if depth == 0 && !only_captures {
            return Move::evaluate(self.evaluate_position());
            // return self.search(depth - 1, alpha, beta, parent, true, &rx);
        }

        // if depth == 0 {
        //     return Move::evaluate(self.evaluate_position());
        // }

        if only_captures && depth < -10 {
            // return Move{ score: 0, action: Action::Evaluation {score: self.evaluate_position()}};
            return Move::evaluate(self.evaluate_position());
        }

        let mut moves = self.collect_all_moves(self.current_color(), only_captures, false);
        if moves.is_empty() {
            return Move::evaluate(self.evaluate_position());
        }

        self.sort_moves(&mut moves);

        // let moves = if only_captures {
        //     &mut moves
        // } else {
        //     parent.children = moves;
        //     &mut parent.children
        // };

        let moves = &mut moves;

        for m in moves {
            self.push_move(m.m);
            let test_move = -self.search(depth - 1, -beta, -alpha, m, only_captures, &rx);
            // println!("test move: {}", test_move.evaluation);
            self.pop_move();

            if test_move.score >= beta.score {
                // println!("Pruning");
                return beta;
            }

            if test_move.score > alpha.score {
                alpha = m.m;
                alpha.score = test_move.score;
            }
        }

        return alpha;
    }

    pub fn white_pieces(&self) -> [Option<Piece>; 16] {
        self.white_pieces.clone()
    }

    pub fn black_pieces(&self) -> [Option<Piece>; 16] {
        self.black_pieces.clone()
    }

    pub fn current_color(&self) -> Color {
        if self.move_stack.len() % 2 == 0 {
            Color::White
        } else {
            Color::Black
        }
    }

    pub fn piece_at(&self, position: &Position) -> &Option<Piece> {
        &self.cells[position.x as usize][position.y as usize].piece
    }

    pub fn evaluate_position(&mut self) -> i32 {
        // self.compute_attacked_cells();
        let mut black_value = {
            let mut sum: i32 = 0;
            for piece in self.black_pieces {
                if let Some(piece) = piece {
                    sum += piece.value();
                }
            }
            sum
        };
        let mut white_value = {
            let mut sum: i32 = 0;
            for piece in self.white_pieces {
                if let Some(piece) = piece {
                    sum += piece.value();
                }
            }
            sum
        };
        // let black_value = if black_value != 0 {black_value} else {-100000};
        // let white_value = if white_value != 0 {white_value} else {-100000};

        // let mut white_activity = 0;
        // let mut black_activity = 0;
        // for y in 0..8 {
        //     for x in 0..8 {
        //         white_activity += self.cell_at(Position::new(x, y)).attacking_white_pieces as i32;
        //         black_activity += self.cell_at(Position::new(x, y)).attacking_black_pieces as i32;
        //     }
        // }

        // white_value += white_activity;
        // black_value += black_activity;

        let perspective = if self.current_color() == Color::White {
            1
        } else {
            -1
        };
        (white_value - black_value) * perspective
    }

    // black or white: 2 values -> 1 bit
    // nothing, pawn, rook, knight, bishop, queen, king: 7 values -> 3 bits
    // 4 bits per cell, 64*4 = 256 bits total per board position
    pub fn position_hash(&self) -> (u64, u64, u64, u64) {
        let hash = |rows: &[[Cell; 8]]| {
            let mut value: u64 = 0;
            for row in rows {
                for cell in row {
                    value <<= 4;
                    value *= 2 << 4;
                    if let Some(piece) = &cell.piece {
                        value += if piece.color == Color::White { 1 } else { 0 };
                        value += piece.t as u64;
                    }
                }
            }
            value
        };

        let v1 = hash(&self.cells[0..2]);
        let v2 = hash(&self.cells[0..4]);
        let v3 = hash(&self.cells[0..6]);
        let v4 = hash(&self.cells[0..8]);

        (v1, v2, v3, v4)
    }

    pub fn compute_attacked_cells(&mut self) {
        for y in 0..8 {
            for x in 0..8 {
                self.cell_mut_at(Position::new(x, y)).attacking_white_pieces = 0;
                self.cell_mut_at(Position::new(x, y)).attacking_black_pieces = 0;
            }
        }

        let white_moves = self.collect_all_moves(Color::White, false, true);
        for m in &white_moves {
            match &m.m.action {
                Action::Evaluation { .. } => {}
                Action::Move { to, .. } => {
                    self.cell_mut_at(to.position).attacking_white_pieces += 1;
                }
                Action::Capture { piece, target } => {
                    self.cell_mut_at(target.position).attacking_white_pieces += 1;
                }
                Action::Promote { .. } => {}
            }
        }

        let black_moves = self.collect_all_moves(Color::Black, false, true);
        for m in &black_moves {
            match &m.m.action {
                Action::Evaluation { .. } => {}
                Action::Move { to, .. } => {
                    self.cell_mut_at(to.position).attacking_black_pieces += 1;
                }
                Action::Capture { piece, target } => {
                    self.cell_mut_at(target.position).attacking_black_pieces += 1;
                }
                Action::Promote { .. } => {}
            }
        }
    }

    pub fn collect_all_moves(
        &self,
        color: Color,
        only_captures: bool,
        include_control: bool,
    ) -> Vec<MoveNode> {
        let mut moves = Vec::new();
        let pieces = match color {
            Color::White => &self.white_pieces[0..self.used_white_pieces],
            Color::Black => &self.black_pieces[0..self.used_black_pieces],
        };

        for piece in pieces {
            if let Some(piece) = piece {
                self.append_piece_moves(&piece, &mut moves, only_captures, include_control);
            }
        }

        moves
    }

    pub fn append_piece_moves(
        &self,
        piece: &Piece,
        moves: &mut Vec<MoveNode>,
        only_captures: bool,
        include_control: bool,
    ) {
        match piece.t {
            Type::Pawn => self.append_pawn_moves(&piece, moves, only_captures, include_control),
            Type::Rook => self.append_rook_moves(&piece, moves, only_captures),
            Type::Bishop => self.append_bishop_moves(&piece, moves, only_captures),
            Type::King => self.append_king_moves(&piece, moves, only_captures),
            Type::Queen => {
                self.append_rook_moves(&piece, moves, only_captures);
                self.append_bishop_moves(&piece, moves, only_captures);
            }
            Type::Knight => self.append_knight_moves(&piece, moves, only_captures),
        }
    }

    pub fn append_knight_moves(
        &self,
        piece: &Piece,
        moves: &mut Vec<MoveNode>,
        only_captures: bool,
    ) {
        let dx = [1, -1, -2, -2, -1, 1, 2, 2];
        let dy = [2, 2, 1, -1, -2, -2, -1, 1];

        for (dx, dy) in zip(dx, dy) {
            let x = piece.position.x + dx;
            let y = piece.position.y + dy;

            if x < 0 || x > 7 || y < 0 || y > 7 {
                continue;
            }

            let position = Position::new(x, y);

            if let Some(target) = self.piece_at(&position) {
                if target.color != piece.color {
                    moves.push(Move::capture_piece(*piece, *target).into());
                }
            } else if !only_captures {
                moves.push(Move::move_piece(*piece, position).into());
            }
        }
    }

    pub fn append_king_moves(&self, piece: &Piece, moves: &mut Vec<MoveNode>, only_captures: bool) {
        let mut try_position = |position| {
            if let Some(target) = self.piece_at(&position) {
                if target.color != piece.color {
                    moves.push(Move::capture_piece(*piece, *target).into());
                }
            } else if !only_captures {
                moves.push(Move::move_piece(*piece, position).into());
            }
        };
        if let Some(position) = piece.position.left(1) {
            try_position(position);
        }

        if let Some(position) = piece.position.up_left(1) {
            try_position(position);
        }

        if let Some(position) = piece.position.up(1) {
            try_position(position);
        }

        if let Some(position) = piece.position.up_right(1) {
            try_position(position);
        }

        if let Some(position) = piece.position.right(1) {
            try_position(position);
        }

        if let Some(position) = piece.position.down_right(1) {
            try_position(position);
        }

        if let Some(position) = piece.position.down(1) {
            try_position(position);
        }

        if let Some(position) = piece.position.down_left(1) {
            try_position(position);
        }
    }

    pub fn append_rook_moves(&self, piece: &Piece, moves: &mut Vec<MoveNode>, only_captures: bool) {
        for x in piece.position.x + 1..=7 {
            if let Some(target) = self.piece_at(&Position::new(x, piece.position.y)) {
                if target.color != piece.color {
                    moves.push(Move::capture_piece(*piece, *target).into());
                }
                break;
            } else if !only_captures {
                moves.push(Move::move_piece(*piece, Position::new(x, piece.position.y)).into());
            }
        }

        for x in (0..piece.position.x).rev() {
            if let Some(target) = self.piece_at(&Position::new(x, piece.position.y)) {
                if target.color != piece.color {
                    moves.push(Move::capture_piece(*piece, *target).into());
                }
                break;
            } else if !only_captures {
                moves.push(Move::move_piece(*piece, Position::new(x, piece.position.y)).into());
            }
        }

        for y in piece.position.y + 1..=7 {
            if let Some(target) = self.piece_at(&Position::new(piece.position.x, y)) {
                if target.color != piece.color {
                    moves.push(Move::capture_piece(*piece, *target).into());
                }
                break;
            } else if !only_captures {
                moves.push(Move::move_piece(*piece, Position::new(piece.position.x, y)).into());
            }
        }

        for y in (0..piece.position.y).rev() {
            if let Some(target) = self.piece_at(&Position::new(piece.position.x, y)) {
                if target.color != piece.color {
                    moves.push(Move::capture_piece(*piece, *target).into());
                }
                break;
            } else if !only_captures {
                moves.push(Move::move_piece(*piece, Position::new(piece.position.x, y)).into());
            }
        }
    }

    pub fn append_bishop_moves(
        &self,
        piece: &Piece,
        moves: &mut Vec<MoveNode>,
        only_captures: bool,
    ) {
        // up - right
        for n in 1..=i8::min(7 - piece.position.x, 7 - piece.position.y) {
            let x = piece.position.x + n;
            let y = piece.position.y + n;
            if let Some(target) = self.piece_at(&Position::new(x, y)) {
                if target.color != piece.color {
                    moves.push(Move::capture_piece(*piece, *target).into());
                }
                break;
            } else if !only_captures {
                moves.push(Move::move_piece(*piece, Position::new(x, y)).into());
            }
        }

        // up - left
        for n in 1..=i8::min(piece.position.x, 7 - piece.position.y) {
            let x = piece.position.x - n;
            let y = piece.position.y + n;
            if let Some(target) = self.piece_at(&Position::new(x, y)) {
                if target.color != piece.color {
                    moves.push(Move::capture_piece(*piece, *target).into());
                }
                break;
            } else if !only_captures {
                moves.push(Move::move_piece(*piece, Position::new(x, y)).into());
            }
        }

        // down - left
        for n in 1..=i8::min(piece.position.x, piece.position.y) {
            let x = piece.position.x - n;
            let y = piece.position.y - n;
            if let Some(target) = self.piece_at(&Position::new(x, y)) {
                if target.color != piece.color {
                    moves.push(Move::capture_piece(*piece, *target).into());
                }
                break;
            } else if !only_captures {
                moves.push(Move::move_piece(*piece, Position::new(x, y)).into());
            }
        }

        // down - right
        for n in 1..=i8::min(7 - piece.position.x, piece.position.y) {
            let x = piece.position.x + n;
            let y = piece.position.y - n;
            if let Some(target) = self.piece_at(&Position::new(x, y)) {
                if target.color != piece.color {
                    moves.push(Move::capture_piece(*piece, *target).into());
                }
                break;
            } else if !only_captures {
                moves.push(Move::move_piece(*piece, Position::new(x, y)).into());
            }
        }
    }

    pub fn append_pawn_moves(
        &self,
        piece: &Piece,
        moves: &mut Vec<MoveNode>,
        only_captures: bool,
        include_control: bool,
    ) {
        match piece.color {
            Color::White => {
                if let Some(position) = piece.position.up_left(1) {
                    if let Some(target) = self.piece_at(&position) {
                        if target.color != piece.color {
                            moves.push(Move::capture_piece(*piece, *target).into());
                        }
                    } else if include_control {
                        moves.push(Move::move_piece(*piece, position).into());
                    }
                }

                if let Some(position) = piece.position.up_right(1) {
                    if let Some(target) = self.piece_at(&position) {
                        if target.color != piece.color {
                            moves.push(Move::capture_piece(*piece, *target).into());
                        }
                    } else if include_control {
                        moves.push(Move::move_piece(*piece, position).into());
                    }
                }

                // promotes
                if piece.position.y == 6 {
                    if let None = self.piece_at(&piece.position.up(1).unwrap()) {
                        moves.push(
                            Move::promote(*piece, piece.position.up(1).unwrap(), Type::Queen).into(),
                        );
                    }
                }

                if !only_captures && !include_control {
                    if let Some(position) = piece.position.up(1) {
                        if let None = self.piece_at(&position) {
                            moves.push(Move::move_piece(*piece, position).into());
                        }
                    }

                    if piece.position.y == 1 {
                        if let None = self.piece_at(&piece.position.up(1).unwrap()) {
                            let target = piece.position.up(2).unwrap();
                            if let None = self.piece_at(&target) {
                                moves.push(Move::move_piece(*piece, target).into());
                            }
                        }
                    }
                }
            }

            Color::Black => {
                if let Some(position) = piece.position.down_left(1) {
                    if let Some(target) = self.piece_at(&position) {
                        if target.color != piece.color {
                            moves.push(Move::capture_piece(*piece, *target).into());
                        }
                    } else if include_control {
                        moves.push(Move::move_piece(*piece, position).into());
                    }
                }

                if let Some(position) = piece.position.down_right(1) {
                    if let Some(target) = self.piece_at(&position) {
                        if target.color != piece.color {
                            moves.push(Move::capture_piece(*piece, *target).into());
                        }
                    } else if include_control {
                        moves.push(Move::move_piece(*piece, position).into());
                    }
                }

                // promotes
                if piece.position.y == 1 {
                    if let None = self.piece_at(&piece.position.down(1).unwrap()) {
                        moves.push(
                            Move::promote(*piece, piece.position.down(1).unwrap(), Type::Queen)
                                .into(),
                        );
                    }
                }

                if !only_captures && !include_control {
                    if let Some(position) = piece.position.down(1) {
                        if let None = self.piece_at(&position) {
                            moves.push(Move::move_piece(*piece, position).into());
                        }
                    }

                    if piece.position.y == 6 {
                        if let None = self.piece_at(&piece.position.down(1).unwrap()) {
                            let target = piece.position.down(2).unwrap();
                            if let None = self.piece_at(&target) {
                                moves.push(Move::move_piece(*piece, target).into());
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn sort_moves(&self, moves: &mut Vec<MoveNode>) {
        moves.sort_by(|lh, rh| {
            let v1 = lh.m.value();
            let v2 = rh.m.value();
            if v1 > v2 {
                Ordering::Less
            } else if v2 > v1 {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
    }

    pub fn cell_mut_at(&mut self, position: Position) -> &mut Cell {
        &mut self.cells[position.x as usize][position.y as usize]
    }

    pub fn cell_at(&self, position: Position) -> &Cell {
        &self.cells[position.x as usize][position.y as usize]
    }

    fn remove_piece_attack(&mut self, piece: &Piece) {
        let mut moves = Vec::with_capacity(21);
        self.append_piece_moves(piece, &mut moves, false, true);

        if piece.color == Color::White {
            for m in &moves {
                match &m.m.action {
                    Action::Evaluation { .. } => {}
                    Action::Move { to, .. } => {
                        self.cell_mut_at(to.position).attacking_white_pieces -= 1
                    }
                    Action::Capture { piece, target } => {
                        self.cell_mut_at(target.position).attacking_white_pieces -= 1
                    }
                    Action::Promote { .. } => {}
                }
            }
        } else {
            for m in &moves {
                match &m.m.action {
                    Action::Evaluation { .. } => {}
                    Action::Move { to, .. } => {
                        self.cell_mut_at(to.position).attacking_black_pieces -= 1
                    }
                    Action::Capture { piece, target } => {
                        self.cell_mut_at(target.position).attacking_black_pieces -= 1
                    }
                    Action::Promote { .. } => {}
                }
            }
        }
    }

    fn add_piece_attack(&mut self, piece: &Piece) {
        let mut moves = Vec::with_capacity(21);
        self.append_piece_moves(piece, &mut moves, false, true);

        if piece.color == Color::White {
            for m in &moves {
                match &m.m.action {
                    Action::Evaluation { .. } => {}
                    Action::Move { to, .. } => {
                        self.cell_mut_at(to.position).attacking_white_pieces += 1
                    }
                    Action::Capture { piece, target } => {
                        self.cell_mut_at(target.position).attacking_white_pieces += 1
                    }
                    Action::Promote { .. } => {}
                }
            }
        } else {
            for m in &moves {
                match &m.m.action {
                    Action::Evaluation { .. } => {}
                    Action::Move { to, .. } => {
                        self.cell_mut_at(to.position).attacking_black_pieces += 1
                    }
                    Action::Capture { piece, target } => {
                        self.cell_mut_at(target.position).attacking_black_pieces += 1
                    }
                    Action::Promote { .. } => {}
                }
            }
        }
    }

    pub fn move_piece(&mut self, from: Piece, to: Piece) {
        assert!(self.cell_at(to.position).piece.is_none());

        self.cell_mut_at(to.position).piece = Some(to);
        self.cell_mut_at(from.position).piece = None;

        debug_assert_eq!(from.index, to.index);

        match from.color {
            Color::White => {
                self.white_pieces[from.index as usize] = Some(to);
            }
            Color::Black => {
                self.black_pieces[from.index as usize] = Some(to);
            }
        }
    }

    fn remove_piece(&mut self, piece: Piece) {
        self.cell_mut_at(piece.position).piece = None;
        match piece.color {
            Color::White => {
                self.white_pieces[piece.index as usize] = None;
                self.white_piece_count -= 1;
            }
            Color::Black => {
                self.black_pieces[piece.index as usize] = None;
                self.black_piece_count -= 1;
            }
        }
    }

    fn add_piece(&mut self, piece: Piece) {
        self.cell_mut_at(piece.position).piece = Some(piece);
        match piece.color {
            Color::White => {
                self.white_pieces[piece.index as usize] = Some(piece);
                self.white_piece_count += 1;
            }
            Color::Black => {
                self.black_pieces[piece.index as usize] = Some(piece);
                self.black_piece_count += 1;
            }
        }
    }

    pub fn make_move(&mut self, m: Move) {
        match m.action {
            Action::Evaluation { .. } => {
                unreachable!()
            }
            Action::Move { from, to } => {
                // self.remove_piece_attack(&from);
                self.move_piece(from, to);
                // self.add_piece_attack(&to);
            }
            Action::Capture { piece, target } => {
                // self.remove_piece_attack(&target);
                // self.remove_piece_attack(&piece);
                self.remove_piece(target);
                self.move_piece(piece, piece.moved(target.position));
                // self.add_piece_attack(&piece.moved(target.position));
            }
            Action::Promote {
                old_piece,
                new_piece,
            } => {
                // self.remove_piece_attack(&old_piece);
                self.move_piece(old_piece, new_piece);
                // self.add_piece_attack(&new_piece);
            }
        }
    }

    pub fn unmake_move(&mut self, m: Move) {
        match m.action {
            Action::Evaluation { .. } => unreachable!(),
            Action::Move { from, to } => {
                // self.remove_piece_attack(&to);
                self.move_piece(to, from);
                // self.add_piece_attack(&from);
            }
            Action::Capture { piece, target } => {
                // self.remove_piece_attack(&piece.moved(target.position));
                self.move_piece(piece.moved(target.position), piece);
                self.add_piece(target);
                // self.add_piece_attack(&target);
                // self.add_piece_attack(&piece);
            }
            Action::Promote {
                old_piece,
                new_piece,
            } => {
                // self.remove_piece_attack(&new_piece);
                self.move_piece(new_piece, old_piece);
                // self.add_piece_attack(&old_piece);
            }
        }
    }

    pub fn push_move(&mut self, m: Move) {
        self.move_stack.push(m);
        self.make_move(m);
        self.move_count += 1;
    }

    pub fn pop_move(&mut self) {
        let m = self.move_stack.pop().unwrap();
        self.unmake_move(m);
    }

    pub fn print_attacked_cells(&self) {
        for y in 0..8 {
            for x in 0..8 {
                print!(
                    "{},{}|",
                    self.cell_at(Position::new(x, 7 - y)).attacking_white_pieces,
                    self.cell_at(Position::new(x, 7 - y)).attacking_black_pieces
                );
            }
            println!();
        }
    }

    pub fn print(&mut self) {
        let mut stdout = termcolor::StandardStream::stdout(ColorChoice::Always);
        let print_line =
            |out: &mut termcolor::StandardStream| writeln!(out, "  {}+", "+---".repeat(8));
        print_line(&mut stdout);
        for y in 0..8 {
            write!(&mut stdout, "{} ", 8 - y);
            for x in 0..8 {
                if let Some(piece) = self.piece_at(&Position::new(x, 7 - y)) {
                    stdout.reset();
                    write!(&mut stdout, "| ");
                    match piece.color {
                        Color::White => stdout.set_color(&self.green),
                        Color::Black => stdout.set_color(&self.red),
                    };
                    write!(&mut stdout, "{}", piece.character());
                    stdout.reset();
                    write!(&mut stdout, " ");
                    // print!(" {} ", piece.character())
                } else {
                    write!(&mut stdout, "|   ");
                }
            }
            println!("|");
            print_line(&mut stdout);
        }
        println!("    a   b   c   d   e   f   g   h");
        println!("position: {}", self.evaluate_position());
    }

    pub fn depth(&self) -> usize {
        self.move_stack.len()
    }
}
