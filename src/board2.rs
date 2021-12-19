pub use super::piece2::*;

#[derive(Clone)]
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
}

#[derive(Clone)]
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

    #[derive(Clone)]
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

    #[derive(Clone)]
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

#[derive(Clone)]
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
            for p in self.pieces.white() {
                if p.active() {
                    (moves, buffer) = self.insert_piece_moves(p, buffer);
                }
            }
        }

        (moves, buffer)
    }

    pub fn insert_piece_moves<'a>(&self, piece: &Piece, buffer: &'a mut [Move]) -> (&'a mut [Move], &'a mut [Move]) {
        match piece.t() {
            Type::Pawn => self.insert_pawn_moves(piece, buffer),
            _ => buffer.split_at_mut(0)
        }
    }

    pub fn insert_pawn_moves<'a>(&self, pawn: &Piece, buffer: &'a mut [Move]) -> (&'a mut [Move], &'a mut [Move]) {
        buffer.split_at_mut(0)
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