use crate::piece::{Color, Piece, Type};
use crate::utils::Position;
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug)]
pub enum Move {
    Move { from: Position, to: Position },
    Take { from: Piece, to: Piece },
    // Promote{from: Piece, to: Piece},
    // TakeAndPromote{target: PieceID, from: Piece, to: Piece}
}

#[derive(Clone)]
pub struct Board {
    white_pieces: [Option<Piece>; 16],
    black_pieces: [Option<Piece>; 16],

    cells: [[Option<Piece>; 8]; 8],
    move_stack: Vec<Move>,
}

impl Board {
    pub fn new_pawn_game() -> Self {
        let mut white_pieces = [None; 16];
        let mut black_pieces = [None; 16];

        let mut cells = [[None; 8]; 8];

        for i in 0..8 {
            let piece = Piece {
                t: Type::Pawn,
                position: Position::new(i as i8, 1),
                index: i,
                color: Color::White,
            };
            white_pieces[i as usize] = Some(piece);
            cells[i as usize][1] = Some(piece);
        }

        for i in 0..8 {
            let piece = Piece {
                t: Type::Pawn,
                position: Position::new(i as i8, 6),
                index: i,
                color: Color::Black,
            };
            black_pieces[i as usize] = Some(piece);
            cells[i as usize][6] = Some(piece);
        }

        Self {
            white_pieces,
            black_pieces,
            cells,
            move_stack: Vec::new(),
        }
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
                    sum += piece.t.value();
                }
            }
            sum
        };
        let white_value = {
            let mut sum: i64 = 0;
            for piece in self.white_pieces {
                if let Some(piece) = piece {
                    sum += piece.t.value();
                }
            }
            sum
        };
        white_value - black_value
    }

    pub fn collect_all_moves(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();
        let pieces = match color {
            Color::White => &self.white_pieces,
            Color::Black => &self.black_pieces,
        };
        for piece in pieces {
            if let Some(piece) = piece {
                self.append_piece_moves(&piece, &mut moves);
            }
        }

        moves
    }

    pub fn append_piece_moves(&self, piece: &Piece, moves: &mut Vec<Move>) {
        match piece.t {
            Type::Pawn => self.append_pawn_moves(&piece, moves),
        }
    }

    pub fn append_pawn_moves(&self, piece: &Piece, moves: &mut Vec<Move>) {
        match piece.color {
            Color::White => {
                if let Some(position) = piece.position.up_left(1) {
                    if let Some(target) = self.piece_at(&position) {
                        if target.color != piece.color {
                            moves.push(Move::Take {
                                from: *piece,
                                to: *target,
                            });
                        }
                    }
                }

                if let Some(position) = piece.position.up_right(1) {
                    if let Some(target) = self.piece_at(&position) {
                        if target.color != piece.color {
                            moves.push(Move::Take {
                                from: *piece,
                                to: *target,
                            });
                        }
                    }
                }

                if let Some(position) = piece.position.up(1) {
                    if let None = self.piece_at(&position) {
                        moves.push(Move::Move {
                            from: piece.position,
                            to: position,
                        });
                    }
                }

                if piece.position.y == 1 {
                    if let None = self.piece_at(&piece.position.up(1).unwrap()) {
                        let target = piece.position.up(2).unwrap();
                        if let None = self.piece_at(&target) {
                            moves.push(Move::Move {
                                from: piece.position,
                                to: target,
                            });
                        }
                    }
                }
            }

            Color::Black => {
                if let Some(position) = piece.position.down_left(1) {
                    if let Some(target) = self.piece_at(&position) {
                        if target.color != piece.color {
                            moves.push(Move::Take {
                                from: *piece,
                                to: *target,
                            });
                        }
                    }
                }

                if let Some(position) = piece.position.down_right(1) {
                    if let Some(target) = self.piece_at(&position) {
                        if target.color != piece.color {
                            let m = Move::Take {
                                from: *piece,
                                to: *target,
                            };
                            moves.push(m);
                        }
                    }
                }

                if let Some(position) = piece.position.down(1) {
                    if let None = self.piece_at(&position) {
                        moves.push(Move::Move {
                            from: piece.position,
                            to: position,
                        });
                    }
                }

                if piece.position.y == 6 {
                    if let None = self.piece_at(&piece.position.down(1).unwrap()) {
                        let target = piece.position.down(2).unwrap();
                        if let None = self.piece_at(&target) {
                            moves.push(Move::Move {
                                from: piece.position,
                                to: target,
                            });
                        }
                    }
                }
            }
        }
    }

    pub fn sort_moves(&self, moves: &mut Vec<Move>) {
        moves.sort_by(|lh, rh| match (lh, rh) {
            (Move::Move { .. }, Move::Move { .. }) => Ordering::Equal,
            (Move::Move { .. }, Move::Take { .. }) => Ordering::Greater,
            (Move::Take { .. }, Move::Move { .. }) => Ordering::Less,
            (Move::Take { to: to1, .. }, Move::Take { to: to2, .. }) => {
                if to1.t.value() > to2.t.value() {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
        });
    }

    pub fn cell_at(&mut self, position: Position) -> &mut Option<Piece> {
        &mut self.cells[position.x as usize][position.y as usize]
    }

    pub fn move_piece(&mut self, from: Position, to: Position) {
        let mut piece = self.cell_at(from).unwrap();
        piece.position = to;
        *self.cell_at(to) = Some(piece);
        *self.cell_at(from) = None;

        match piece.color {
            Color::White => {
                self.white_pieces[piece.index as usize] = Some(piece);
            }
            Color::Black => {
                self.black_pieces[piece.index as usize] = Some(piece);
            }
        }
    }

    pub fn remove_piece(&mut self, piece: Piece) {
        *self.cell_at(piece.position) = None;
        match piece.color {
            Color::White => {
                self.white_pieces[piece.index as usize] = None;
            }
            Color::Black => {
                self.black_pieces[piece.index as usize] = None;
            }
        }
    }

    pub fn add_piece(&mut self, piece: Piece) {
        *self.cell_at(piece.position) = Some(piece);
        match piece.color {
            Color::White => {
                self.white_pieces[piece.index as usize] = Some(piece);
            }
            Color::Black => {
                self.black_pieces[piece.index as usize] = Some(piece);
            }
        }
    }

    pub fn push_move(&mut self, m: Move) {
        self.move_stack.push(m);
        match m {
            Move::Move { from, to } => {
                self.move_piece(from, to);
            }
            Move::Take { from, to } => {
                self.remove_piece(to);
                self.move_piece(from.position, to.position);
            }
        }
    }

    pub fn pop_move(&mut self) {
        let m = self.move_stack.pop().unwrap();
        match m {
            Move::Move { from, to } => {
                self.move_piece(to, from);
            }
            Move::Take { from, to } => {
                self.move_piece(to.position, from.position);
                self.add_piece(to);
            }
        }
    }

    pub fn print(&self) {
        for y in 0..8 {
            for x in 0..8 {
                if let Some(piece) = self.piece_at(&Position::new(x, 7 - y)) {
                    match piece.color {
                        Color::White => print!("x "),
                        Color::Black => print!("o "),
                    }
                } else {
                    print!(". ");
                }
            }
            println!();
        }
        println!();
    }

    pub fn depth(&self) -> usize {
        self.move_stack.len()
    }
}
