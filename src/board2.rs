use std::fmt::{Debug, Display, Formatter};
pub use super::piece2::*;

#[derive(Copy, Clone)]
pub struct Move {
    action: Action,
    piece_ref: PieceRef,
    score: i32,
}

impl Move {
    pub fn none() -> Move {
        Self {
            action: Action::None,
            piece_ref: PieceRef::null(),
            score: 0,
        }
    }

    pub fn new_move(piece_ref: PieceRef, start: Position, end: Position) -> Move {
        Move {
            piece_ref,
            action: Action::Move { start, end },
            score: 0,
        }
    }

    pub fn new_capture(piece_ref: PieceRef, target_ref: PieceRef) -> Move {
        Move {
            piece_ref,
            action: Action::Capture { target: target_ref },
            score: 0,
        }
    }
}

#[derive(Copy, Clone)]
enum Action {
    None,
    Move {
        start: Position,
        end: Position,
    },

    Capture {
        target: PieceRef,
    },

    Promote {
        t: Type,
        end: Position,
    },

    CaptureAndPromote {
        target: PieceRef,
        t: Type,
    },

    Castle, // action piece is the tower used for this
}

mod utils {
    use std::ops::{Index, IndexMut};
    use crate::board2::{PieceRef, Position, Piece};

    #[derive(PartialEq, Eq, Clone)]
    pub struct PieceList {
        pieces: [super::Piece; 32],
        used_white_pieces: usize,
        used_black_pieces: usize,
    }

    impl PieceList {
        pub fn new() -> Self {
            Self {
                pieces: [Piece::null(); 32],
                used_white_pieces: 0,
                used_black_pieces: 0,
            }
        }

        pub fn add_new_piece(&mut self, piece: Piece) -> PieceRef {
            let color = piece.color();
            PieceRef::new(color, self.used_white_pieces);
            if color == super::Color::White {
                assert!(self.used_white_pieces < 16);
                self.pieces[self.used_white_pieces] = piece;
                self.used_white_pieces += 1;
                PieceRef::new(color, self.used_white_pieces - 1)
            } else {
                assert!(self.used_black_pieces < 16);
                self.pieces[self.used_black_pieces] = piece;
                self.used_black_pieces += 1;
                PieceRef::new(color, self.used_black_pieces - 1)
            }
        }

        pub fn white(&self) -> &[Piece] {
            &self.pieces[0..16]
        }

        pub fn black(&self) -> &[Piece] {
            &self.pieces[16..32]
        }
    }

    impl Index<PieceRef> for PieceList {
        type Output = Piece;

        fn index(&self, piece_ref: PieceRef) -> &Self::Output {
            &self.pieces[piece_ref.index()]
        }
    }

    impl IndexMut<PieceRef> for PieceList {
        fn index_mut(&mut self, piece_ref: PieceRef) -> &mut Self::Output {
            &mut self.pieces[piece_ref.index()]
        }
    }

    #[derive(PartialEq, Eq, Clone)]
    pub struct PieceBoard {
        piece_refs: [PieceRef; 64],
    }

    impl PieceBoard {
        pub fn new() -> Self {
            Self {
                piece_refs: [PieceRef::null(); 64]
            }
        }
    }

    impl Index<Position> for PieceBoard {
        type Output = PieceRef;

        fn index(&self, position: Position) -> &Self::Output {
            &self.piece_refs[position.index()]
        }
    }

    impl IndexMut<Position> for PieceBoard {
        fn index_mut(&mut self, position: Position) -> &mut Self::Output {
            &mut self.piece_refs[position.index()]
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct Board {
    board: utils::PieceBoard,
    pieces: utils::PieceList,
}

impl Board {
    pub fn new() -> Self {
        Self {
            board: utils::PieceBoard::new(),
            pieces: utils::PieceList::new(),
        }
    }

    pub fn add_new_piece(&mut self, color: Color, t: Type, x: i8, y: i8) {
        let piece = Piece::new(color, t, Position::new(x, y));
        let pref = self.pieces.add_new_piece(piece);
        self.board[piece.position] = pref;
    }

    pub fn piece_at(&self, position: Position) -> Option<&Piece> {
        let pref = self.board[position];
        if pref.active() {
            Some(&self.pieces[pref])
        } else {
            None
        }
    }

    pub fn insert_all_moves<'a>(&self, color: Color, buffer: &'a mut [Move]) -> (&'a mut [Move], &'a mut [Move]) {
        let (mut moves, mut buffer) = buffer.split_at_mut(0);
        if color == Color::White {
            for (i, p) in self.pieces.white().iter().enumerate() {
                let pref = PieceRef::new(color, i);
                if p.active() {
                    (moves, buffer) = self.insert_piece_moves(pref, p, buffer);
                }
            }
        } else {
            for (i, p) in self.pieces.black().iter().enumerate() {
                let pref = PieceRef::new(color, i);
                if p.active() {
                    (moves, buffer) = self.insert_piece_moves(pref, p, buffer);
                }
            }
        }

        (moves, buffer)
    }

    pub fn insert_piece_moves<'a>(&self, pref: PieceRef, piece: &Piece, buffer: &'a mut [Move]) -> (&'a mut [Move], &'a mut [Move]) {
        match piece.t() {
            Type::Pawn => self.insert_pawn_moves(pref, piece, buffer),
            _ => buffer.split_at_mut(0)
        }
    }

    pub fn insert_pawn_moves<'a>(&self, pref: PieceRef, pawn: &Piece, buffer: &'a mut [Move]) -> (&'a mut [Move], &'a mut [Move]) {
        let mut count = 0;
        let p = pawn.position;

        if pawn.color() == Color::White {
            if p.y() < 7 {
                if !self.board[p.dp(0, 1)].active() {
                    buffer[count] = Move::new_move(pref, p, p.dp(0, 1));
                    count += 1;

                    if p.y() == 1 && !self.board[p.dp(0, 2)].active() {
                        buffer[count] = Move::new_move(pref, p, p.dp(0, 2));
                        count += 1;
                    }
                }

                if p.x() > 0 {
                    let target_ref = self.board[p.dp(-1, 1)];
                    if target_ref.active() && target_ref.color() == Color::Black {
                        buffer[count] = Move::new_capture(pref, target_ref);
                        count += 1;
                    }
                }

                if p.x() < 7 {
                    let target_ref = self.board[p.dp(1, 1)];
                    if target_ref.active() && target_ref.color() == Color::Black {
                        buffer[count] = Move::new_capture(pref, target_ref);
                        count += 1;
                    }
                }
            }
        } else {
            if p.y() > 0 {
                if !self.board[p.dp(0, -1)].active() {
                    buffer[count] = Move::new_move(pref, p, p.dp(0, -1));
                    count += 1;

                    if p.y() == 6 && !self.board[p.dp(0, -2)].active() {
                        buffer[count] = Move::new_move(pref, p, p.dp(0, -2));
                        count += 1;
                    }
                }

                if p.x() > 0 {
                    let target_ref = self.board[p.dp(-1, -1)];
                    if target_ref.active() && target_ref.color() == Color::White {
                        buffer[count] = Move::new_capture(pref, target_ref);
                        count += 1;
                    }
                }

                if p.x() < 7 {
                    let target_ref = self.board[p.dp(1, -1)];
                    if target_ref.active() && target_ref.color() == Color::White {
                        buffer[count] = Move::new_capture(pref, target_ref);
                        count += 1;
                    }
                }
            }
        }

        let (mut moves, mut buffer) = buffer.split_at_mut(count);
        (moves, buffer)
    }


    pub fn make_move(&mut self, m: &Move) {
        match m.action {
            Action::Move { start, end } => {
                debug_assert!(self.piece_at(end).is_none());
                debug_assert_eq!(self.piece_at(start).unwrap().position, start);
                self.board[end] = m.piece_ref;
                self.board[start].set_active(false);
                self.pieces[m.piece_ref].position = end;
            }
            Action::Capture { target } => {
                // copy positions
                let start = self.pieces[m.piece_ref].position;
                let end = self.pieces[target].position;

                // swap positions
                self.pieces[m.piece_ref].position = end;
                self.pieces[target].position = start;

                // update board
                self.board[end] = m.piece_ref;
                self.board[start].set_active(false);
            }
            _ => todo!()
        }
    }

    pub fn revert_move(&mut self, m: &Move) {
        match m.action {
            Action::Move { start, end } => {
                debug_assert!(self.piece_at(start).is_none());
                debug_assert_eq!(self.piece_at(end).unwrap().position, end);
                self.board[start] = m.piece_ref;
                self.board[end].set_active(false);
                self.pieces[m.piece_ref].position = start;
            }
            Action::Capture { target } => {
                let end = self.pieces[m.piece_ref].position;
                let start = self.pieces[target].position;

                // swap back positions
                self.pieces[m.piece_ref].position = start;
                self.pieces[target].position = end;

                // update board
                self.board[start] = m.piece_ref;
                self.board[end] = target;
            }
            _ => todo!()
        }
    }
}

// Formatting

impl Debug for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let line = "+---+---+---+---+---+---+---+---+";
        // write!(f, )
        // for pref in self.board {
        //     write!(f, "{}", "a");
        // }
        write!(f, "hello")
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.action {
            Action::None => write!(f, "<None>"),
            Action::Move { start, end } => write!(f, "{} {}", start, end),
            // Action::Capture { target } => {
            //     write!(f, "{} {}", start, end)
            // }
            // Action::Promote { .. } => {}
            // Action::CaptureAndPromote { .. } => {}
            // Action::Castle => {}
            _ => Ok(())
        };
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn add_piece1() {
        use super::{Piece, Position, Color, Type, Board};
        let mut board = Board::new();
        board.add_new_piece(Color::Black, Type::Rook, 5, 6);
        assert!(board.piece_at(Position::new(5, 6)).is_some());
        assert!(board.piece_at(Position::new(5, 7)).is_none());
    }
}