use crate::piece::{Color, Piece, Type};
use crate::utils::Position;
use std::cmp::Ordering;
use termcolor::{ColorChoice, ColorSpec, WriteColor};
use std::io::Write;
use std::ops::Neg;

#[derive(Copy, Clone, Debug)]
pub struct Move {
    evaluation: i64,
    action: Action
}

#[derive(Copy, Clone, Debug)]
pub enum Action {
    EvaluatePosition,
    Move { from: Piece, to: Piece },
    Take { from: Piece, to: Piece},
    Promote{from: Piece, to: Piece},
    // TakeAndPromote{target: PieceID, from: Piece, to: Piece}
}

impl Move {
    pub fn value(&self) -> i64 {
        match self.action {
            Action::EvaluatePosition => self.evaluation,
            Action::Move { from, to } => to.value() - from.value(),
            Action::Take { from, to } => to.value() * 10 - from.value(),
            Action::Promote {to, ..} => to.value()
        }
    }

    pub fn evaluate(evaluation: i64) -> Self {
        Self {
            evaluation,
            action: Action::EvaluatePosition
        }
    }

    pub fn take_piece(from: Piece, to: Piece) -> Self {
        Self {
            evaluation: 0,
            action: Action::Take {from, to}
        }
    }

    pub fn move_piece(from: Piece, to: Position) -> Self {
        let piece = Piece {
            t: from.t,
            position: to,
            color: from.color,
            index: from.index
        };
        Self {
            evaluation: 0,
            action: Action::Move {from, to: piece}
        }
    }

    pub fn promote_piece(from: Piece, to: Position, t: Type) -> Self {
        let piece = Piece {
            t,
            position: to,
            color: from.color,
            index: from.index
        };
        Self {
            evaluation: 0,
            action: Action::Promote {from, to: piece}
        }
    }

    pub fn is_valid(&self) -> bool {
        match self.action {
            Action::EvaluatePosition => false,
            _ => true
        }
    }

    pub fn evaluation(&self) -> i64 {
        self.evaluation
    }
}

impl Neg for Move {
    type Output = Move;

    fn neg(self) -> Self::Output {
        Self {
            action: self.action,
            evaluation: -self.evaluation
        }
    }
}

#[derive(Clone)]
pub struct Board {
    white_pieces: [Option<Piece>; 16],
    black_pieces: [Option<Piece>; 16],

    used_white_pieces: usize,
    used_black_pieces: usize,

    cells: [[Option<Piece>; 8]; 8],
    move_stack: Vec<Move>,

    green: ColorSpec,
    red: ColorSpec,

    white_piece_count: i8,
    black_piece_count: i8
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
            cells: [[None; 8]; 8],
            move_stack: Vec::new(),
            green,
            red,
            used_black_pieces: 0,
            used_white_pieces: 0,
            white_piece_count: 0,
            black_piece_count: 0
        }
    }

    pub fn new_promote_game() -> Self {
        let mut game = Board::new_empty_game();

        game.add_new_piece(Color::White, Type::Pawn, 0, 1);
        game.add_new_piece(Color::Black, Type::Pawn, 7, 6);

        game
    }

    pub fn new_classic_game() -> Self {
        let mut game = Board::new_pawn_game();

        game.add_new_piece(Color::White, Type::Rook, 0, 0);
        game.add_new_piece(Color::White, Type::Rook, 7, 0);

        game.add_new_piece(Color::Black, Type::Rook, 0, 7);
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
            self.white_pieces[self.used_white_pieces] = Some(Piece{
                t,
                position: Position { x, y },
                color,
                index: self.used_white_pieces as u8
            });
            self.cells[x as usize][y as usize] = self.white_pieces[self.used_white_pieces];
            self.used_white_pieces += 1;
        }
        else {
            self.black_pieces[self.used_black_pieces] = Some(Piece{
                t,
                position: Position { x, y },
                color,
                index: self.used_black_pieces as u8
            });
            self.cells[x as usize][y as usize] = self.black_pieces[self.used_black_pieces];
            self.used_black_pieces += 1;
        }
    }

    pub fn search(&mut self, depth: i32, mut alpha: Move, beta: Move, only_captures: bool) -> Move {
        if depth == 0 && !only_captures{
            // return Move::evaluate(self.evaluate_position());
            return self.search(depth - 1, alpha, beta, true);
        }

        if only_captures && depth < 20 {
            return Move::evaluate(self.evaluate_position());
        }

        let mut moves = self.collect_all_moves(self.current_color(), only_captures);
        if moves.is_empty() {
            return Move::evaluate(self.evaluate_position());
        }

        self.sort_moves(&mut moves);

        for m in moves {
            self.push_move(m);
            let test_move = -self.search(depth - 1, -beta, -alpha, only_captures);
            // println!("test move: {}", test_move.evaluation);
            self.pop_move();

            if test_move.evaluation >= beta.evaluation {
                // println!("Pruning");
                return beta;
            }

            if test_move.evaluation > alpha.evaluation {
                alpha = m;
                alpha.evaluation = test_move.evaluation;
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
        &self.cells[position.x as usize][position.y as usize]
    }

    pub fn evaluate_position(&self) -> i64 {
        let black_value = {
            let mut sum: i64 = 0;
            for piece in self.black_pieces {
                if let Some(piece) = piece {
                    sum += piece.value();
                }
            }
            sum
        };
        let white_value = {
            let mut sum: i64 = 0;
            for piece in self.white_pieces {
                if let Some(piece) = piece {
                    sum += piece.value();
                }
            }
            sum
        };
        let perspective = if self.current_color() == Color::White {1}else{-1};
        (white_value - black_value)*perspective
    }

    pub fn collect_all_moves(&self, color: Color, only_captures: bool) -> Vec<Move> {
        let mut moves = Vec::new();
        let pieces = match color {
            Color::White => &self.white_pieces[0..self.used_white_pieces],
            Color::Black => &self.black_pieces[0..self.used_black_pieces],
        };

        for piece in pieces {
            if let Some(piece) = piece {
                self.append_piece_moves(&piece, &mut moves, only_captures);
            }
        }

        moves
    }

    pub fn append_piece_moves(&self, piece: &Piece, moves: &mut Vec<Move>, only_captures: bool) {
        match piece.t {
            Type::Pawn => self.append_pawn_moves(&piece, moves, only_captures),
            Type::Rook => self.append_rook_moves(&piece, moves, only_captures)
        }
    }

    pub fn append_rook_moves(&self, piece: &Piece, moves: &mut Vec<Move>, only_captures: bool) {
        for x in piece.position.x+1..=7 {
            if let Some(target) = self.piece_at(&Position::new(x, piece.position.y)) {
                if target.color != piece.color {
                    moves.push(Move::take_piece(*piece, *target));
                }
                break;
            } else {
                moves.push(Move::move_piece(*piece, Position::new(x, piece.position.y)));
            }
        }

        for x in (0..piece.position.x).rev() {
            if let Some(target) = self.piece_at(&Position::new(x, piece.position.y)) {
                if target.color != piece.color {
                    moves.push(Move::take_piece(*piece, *target));
                }
                break;
            } else {
                moves.push(Move::move_piece(*piece, Position::new(x, piece.position.y)));
            }
        }

        for y in piece.position.y+1..=7 {
            if let Some(target) = self.piece_at(&Position::new(piece.position.x, y)) {
                if target.color != piece.color {
                    moves.push(Move::take_piece(*piece, *target));
                }
                break;
            } else {
                moves.push(Move::move_piece(*piece, Position::new(piece.position.x, y)));
            }
        }

        for y in (0..piece.position.y).rev() {
            if let Some(target) = self.piece_at(&Position::new(piece.position.x, y)) {
                if target.color != piece.color {
                    moves.push(Move::take_piece(*piece, *target));
                }
                break;
            } else {
                moves.push(Move::move_piece(*piece, Position::new(piece.position.x, y)));
            }
        }
    }

    pub fn append_pawn_moves(&self, piece: &Piece, moves: &mut Vec<Move>, only_captures: bool) {
        match piece.color {
            Color::White => {
                if let Some(position) = piece.position.up_left(1) {
                    if let Some(target) = self.piece_at(&position) {
                        if target.color != piece.color {
                            moves.push(Move::take_piece(
                                *piece,
                                *target
                            ));
                        }
                    }
                }

                if let Some(position) = piece.position.up_right(1) {
                    if let Some(target) = self.piece_at(&position) {
                        if target.color != piece.color {
                            moves.push(Move::take_piece(
                                *piece,
                                *target
                            ));
                        }
                    }
                }

                // promotes
                if piece.position.y == 6 {
                    if let None = self.piece_at(&piece.position.up(1).unwrap()) {
                        moves.push(Move::promote_piece(*piece, piece.position.up(1).unwrap(), Type::Rook))
                    }
                }

                if !only_captures {
                    if let Some(position) = piece.position.up(1) {
                        if let None = self.piece_at(&position) {
                            moves.push(Move::move_piece(
                                *piece,
                                position,
                            ));
                        }
                    }

                    if piece.position.y == 1 {
                        if let None = self.piece_at(&piece.position.up(1).unwrap()) {
                            let target = piece.position.up(2).unwrap();
                            if let None = self.piece_at(&target) {
                                moves.push(Move::move_piece(
                                    *piece,
                                    target,
                                ));
                            }
                        }
                    }
                }
            }

            Color::Black => {
                if let Some(position) = piece.position.down_left(1) {
                    if let Some(target) = self.piece_at(&position) {
                        if target.color != piece.color {
                            moves.push(Move::take_piece(
                                *piece,
                                *target
                            ));
                        }
                    }
                }

                if let Some(position) = piece.position.down_right(1) {
                    if let Some(target) = self.piece_at(&position) {
                        if target.color != piece.color {
                            moves.push(Move::take_piece(
                                *piece,
                                *target
                            ));
                        }
                    }
                }

                // promotes
                if piece.position.y == 1 {
                    if let None = self.piece_at(&piece.position.down(1).unwrap()) {
                        moves.push(Move::promote_piece(*piece, piece.position.down(1).unwrap(), Type::Rook))
                    }
                }

                if !only_captures {
                    if let Some(position) = piece.position.down(1) {
                        if let None = self.piece_at(&position) {
                            moves.push(Move::move_piece(
                                *piece,
                                position,
                            ));
                        }
                    }

                    if piece.position.y == 6 {
                        if let None = self.piece_at(&piece.position.down(1).unwrap()) {
                            let target = piece.position.down(2).unwrap();
                            if let None = self.piece_at(&target) {
                                moves.push(Move::move_piece(
                                    *piece,
                                    target,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn sort_moves(&self, moves: &mut Vec<Move>) {
        moves.sort_by(|lh, rh| {
            let v1 = lh.value();
            let v2 = rh.value();
            if v1 > v2 { Ordering::Less}
            else if v2 > v1 { Ordering::Greater }
            else {Ordering::Equal}
        });
    }

    pub fn cell_at(&mut self, position: Position) -> &mut Option<Piece> {
        &mut self.cells[position.x as usize][position.y as usize]
    }

    pub fn move_piece(&mut self, from: Piece, to: Piece) {

        *self.cell_at(to.position) = Some(to);
        *self.cell_at(from.position) = None;

        assert_eq!(from.index, to.index);

        match from.color {
            Color::White => {
                self.white_pieces[from.index as usize] = Some(to);
            }
            Color::Black => {
                self.black_pieces[from.index as usize] = Some(to);
            }
        }
    }

    pub fn remove_piece(&mut self, piece: Piece) {
        *self.cell_at(piece.position) = None;
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

    pub fn add_piece(&mut self, piece: Piece) {
        *self.cell_at(piece.position) = Some(piece);
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
            Action::EvaluatePosition => {
                unreachable!()
            },
            Action::Move { from, to } => {
                self.move_piece(from, to);
            }
            Action::Take { from, to } => {
                self.remove_piece(to);
                self.move_piece(from, from.moved(to.position));
            }
            Action::Promote {from, to} => {
                self.move_piece(from, to);
            }
        }
    }

    pub fn unmake_move(&mut self, m: Move) {
        match m.action {
            Action::EvaluatePosition => unreachable!(),
            Action::Move { from, to } => {
                self.move_piece(to, from);
            }
            Action::Take { from, to } => {
                self.move_piece(from.moved(to.position), from);
                self.add_piece(to);
            }
            Action::Promote {from, to} => {
                self.move_piece(to, from);
            }
        }
    }

    pub fn push_move(&mut self, m: Move) {
        self.move_stack.push(m);
        self.make_move(m);
    }

    pub fn pop_move(&mut self) {
        let m = self.move_stack.pop().unwrap();
        self.unmake_move(m);
    }

    pub fn print(&self) {
        let mut stdout = termcolor::StandardStream::stdout(ColorChoice::Always);
        let print_line = |out: &mut termcolor::StandardStream| writeln!(out, "{}+", "+---".repeat(8));
        print_line(&mut stdout);
        for y in 0..8 {
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
        println!("position: {}", self.evaluate_position());
    }

    pub fn depth(&self) -> usize {
        self.move_stack.len()
    }
}
