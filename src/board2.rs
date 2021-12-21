use std::fmt::{Debug, Display, Formatter};
use std::task::Poll::Pending;
pub use crate::piece2::*;

use bitflags::bitflags;

bitflags! {
    pub struct MoveFlags : u8 {
        const EMPTY = 0b0000;
        const KING_MOVED = 0b0001;
        const KING_ROOK_MOVED = 0b0010;
        const QUEEN_ROOK_MOVED = 0b0100;
        const FULL = 0b0111;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Move {
    pub action: Action,
    pub piece_ref: PieceRef,
    pub score: i32,
    flags: MoveFlags,
    children_offset: usize,
    children_count: usize,
}

impl std::ops::Neg for Move {
    type Output = Move;

    fn neg(self) -> Self::Output {
        Self::Output {
            score: -self.score,
            action: self.action,
            flags: self.flags,
            children_count: self.children_count,
            children_offset: self.children_offset,
            piece_ref: self.piece_ref,
        }
    }
}

impl Move {
    pub fn action(&self) -> &Action {
        &self.action
    }

    pub fn flags(&self) -> &MoveFlags {
        &self.flags
    }

    pub fn none() -> Move {
        Self {
            action: Action::None,
            piece_ref: PieceRef::null(),
            score: 0,
            flags: MoveFlags::EMPTY,
            children_offset: 0,
            children_count: 0,
        }
    }

    pub fn new_move(piece_ref: PieceRef, start: Position, end: Position, flags: MoveFlags) -> Move {
        Move {
            piece_ref,
            action: Action::Move { start, end },
            score: 0,
            flags,
            children_offset: 0,
            children_count: 0,
        }
    }

    pub fn new_capture(piece_ref: PieceRef, target_ref: PieceRef, flags: MoveFlags) -> Move {
        Move {
            piece_ref,
            action: Action::Capture { target: target_ref },
            score: 0,
            flags,
            children_offset: 0,
            children_count: 0,
        }
    }

    pub fn new_promote(piece_ref: PieceRef, t: Type, end: Position) -> Move {
        Move {
            piece_ref,
            action: Action::Promote { t, end },
            score: 0,
            flags: MoveFlags::EMPTY,
            children_offset: 0,
            children_count: 0,
        }
    }

    pub fn new_capture_and_promote(piece_ref: PieceRef, target: PieceRef, t: Type) -> Move {
        Move {
            piece_ref,
            action: Action::CaptureAndPromote { t, target },
            score: 0,
            flags: MoveFlags::EMPTY,
            children_offset: 0,
            children_count: 0,
        }
    }

    pub fn new_castle(king_ref: PieceRef, flags: MoveFlags) -> Move {
        Move {
            piece_ref: king_ref,
            action: Action::Castle,
            score: 0,
            flags,
            children_offset: 0,
            children_count: 0,
        }
    }

    pub fn new_evaluate(score: i32) -> Move {
        Move {
            piece_ref: PieceRef::null(),
            action: Action::Evaluate,
            score,
            flags: MoveFlags::EMPTY,
            children_offset: 0,
            children_count: 0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Action {
    None,
    Evaluate,
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

    Castle, // ony flags are needed for the piece selection
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
            if color == super::Color::White {
                assert!(self.used_white_pieces < 16);
                self.pieces[self.used_white_pieces] = piece;
                self.used_white_pieces += 1;
                PieceRef::new(color, self.used_white_pieces - 1)
            } else {
                assert!(self.used_black_pieces < 16);
                self.pieces[self.used_black_pieces + 16] = piece;
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

        pub fn piece_refs(&self) -> &[PieceRef; 64] {
            &self.piece_refs
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


    impl Index<(i8, i8)> for PieceBoard {
        type Output = PieceRef;

        fn index(&self, xy: (i8, i8)) -> &Self::Output {
            &self.piece_refs[Position::from(xy).index()]
        }
    }

    impl IndexMut<(i8, i8)> for PieceBoard {
        fn index_mut(&mut self, xy: (i8, i8)) -> &mut Self::Output {
            &mut self.piece_refs[Position::from(xy).index()]
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct Board {
    board: utils::PieceBoard,
    pieces: utils::PieceList,
    white_move_flags: MoveFlags,
    black_move_flags: MoveFlags,
}

impl Board {
    pub fn new() -> Self {
        Self {
            board: utils::PieceBoard::new(),
            pieces: utils::PieceList::new(),
            white_move_flags: MoveFlags::EMPTY,
            black_move_flags: MoveFlags::EMPTY,
        }
    }

    pub fn piece_from_ref(&self, pref: PieceRef) -> &Piece {
        &self.pieces[pref]
    }

    // black or white: 2 values -> 1 bit
    // nothing, pawn, rook, knight, bishop, queen, king: 7 values -> 3 bits
    // 4 bits per cell, 64*4 = 256 bits total per board position
    pub fn position_hash(&self) -> (u64, u64, u64, u64) {
        todo!();
        // let hash = |prefs: &[[PieceRef; 8]]| {
        //     let mut value: u64 = 0;
        //     for pref in prefs {
        //         value <<= 4;
        //         value *= 2 << 4;
        //         if let Some(piece) = &cell.piece {
        //             value += if piece.color == Color::White { 1 } else { 0 };
        //             value += piece.t as u64;
        //         }
        //     }
        //     value
        // };
        //
        // let v1 = hash(&self.cells[0..2]);
        // let v2 = hash(&self.cells[0..4]);
        // let v3 = hash(&self.cells[0..6]);
        // let v4 = hash(&self.cells[0..8]);
        //
        // (v1, v2, v3, v4)
    }

    pub fn assert_consistency(&self) {
        let mut white_count = 0;
        for piece in self.pieces.white() {
            if piece.active() {
                white_count += 1;
                assert_eq!(&self.pieces[self.board[piece.position]], piece);
            }
        }

        let mut black_count = 0;
        for piece in self.pieces.black() {
            if piece.active() {
                black_count += 1;
                assert_eq!(&self.pieces[self.board[piece.position]], piece);
            }
        }

        for pref in self.board.piece_refs() {
            if pref.active() {
                assert_eq!(&self.board[self.pieces[*pref].position], pref);
            }
        }
    }

    pub fn black_pieces(&self) -> &[Piece] {
        self.pieces.black()
    }

    pub fn white_pieces(&self) -> &[Piece] {
        self.pieces.white()
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

    pub fn piece_at_xy(&self, x: i8, y: i8) -> Option<&Piece> {
        self.piece_at(Position::new(x, y))
    }

    fn move_piece(&mut self, p1: (i8, i8), p2: (i8, i8)) {
        debug_assert!(!self.board[p2].active());
        let pref = self.board[p1];
        self.board[p2] = pref;
        self.board[p1] = PieceRef::null();

        self.pieces[pref].position = p2.into();
    }

    pub fn insert_all_moves(&self, color: Color, buffer: &mut [Move]) -> usize {
        let mut count = 0;
        if color == Color::White {
            for (i, p) in self.pieces.white().iter().enumerate() {
                let pref = PieceRef::new(color, i);
                if p.active() {
                    count += self.insert_piece_moves(pref, p, &mut buffer[count..]);
                }
            }
        } else {
            for (i, p) in self.pieces.black().iter().enumerate() {
                let pref = PieceRef::new(color, i);
                if p.active() {
                    count += self.insert_piece_moves(pref, p, &mut buffer[count..]);
                }
            }
        }

        count
    }

    pub fn insert_piece_moves(&self, pref: PieceRef, piece: &Piece, buffer: &mut [Move]) -> usize {
        let mut count = 0;
        let flags = if piece.color() == Color::White { &self.white_move_flags } else { &self.black_move_flags };
        match piece.t() {
            Type::Pawn => count += self.insert_pawn_moves(pref, piece, &mut buffer[count..]),
            Type::Rook => count += self.insert_rook_moves(pref, piece, flags, &mut buffer[count..]),
            Type::Bishop => count += self.insert_bishop_moves(pref, piece, &mut buffer[count..]),
            Type::Queen => count += self.insert_queen_moves(pref, piece, &mut buffer[count..]),
            Type::King => count += self.insert_king_moves(pref, piece, flags, &mut buffer[count..]),
            Type::Knight => count += self.insert_knight_moves(pref, piece, &mut buffer[count..]),
            _ => unreachable!()
        }

        count
    }

    pub fn insert_king_moves(&self, pref: PieceRef, king: &Piece, board_flags: &MoveFlags, buffer: &mut [Move]) -> usize {
        let mut count = 0;
        let vdx = [0, -1, -1, -1, 0, 1, 1, 1];
        let vdy = [1, 1, 0, -1, -1, -1, 0, 1];

        let (x0, y0) = king.position.xy();

        let mut flags = MoveFlags::EMPTY;

        if !board_flags.contains(MoveFlags::KING_MOVED) {
            flags.set(MoveFlags::KING_MOVED, true);
        }

        for (dx, dy) in std::iter::zip(vdx, vdy) {
            if x0 + dx < 0 || x0 + dx > 7 || y0 + dy < 0 || y0 + dy > 7 {
                continue;
            }
            let p = Position::new(x0 + dx, y0 + dy);
            let target_ref = self.board[p];
            if target_ref.active() && target_ref.color() != king.color() {
                buffer[count] = Move::new_capture(pref, target_ref, flags);
                count += 1;
            } else if !target_ref.active() {
                buffer[count] = Move::new_move(pref, king.position, p, flags);
                count += 1;
            }
        }

        if !board_flags.contains(MoveFlags::KING_MOVED) {
            if !board_flags.contains(MoveFlags::QUEEN_ROOK_MOVED) &&
                !self.board[(1, y0)].active() &&
                !self.board[(2, y0)].active() &&
                !self.board[(3, y0)].active() {
                let mut flags = flags;
                flags.set(MoveFlags::QUEEN_ROOK_MOVED, true);
                flags.set(MoveFlags::KING_MOVED, true);
                buffer[count] = Move::new_castle(pref, flags);
                count += 1;
            }

            if !board_flags.contains(MoveFlags::KING_ROOK_MOVED) &&
                !self.board[(5, y0)].active() &&
                !self.board[(6, y0)].active() {
                let mut flags = flags;
                flags.set(MoveFlags::KING_ROOK_MOVED, true);
                flags.set(MoveFlags::KING_MOVED, true);
                buffer[count] = Move::new_castle(pref, flags);
                count += 1;
            }
        }


        count
    }

    pub fn insert_knight_moves(&self, pref: PieceRef, knight: &Piece, buffer: &mut [Move]) -> usize {
        let mut count = 0;
        let vdx = [-1, -2, -2, -1, 1, 2, 2, 1];
        let vdy = [2, 1, -1, -2, -2, -1, 1, 2];

        let (x0, y0) = knight.position.xy();

        for (dx, dy) in std::iter::zip(vdx, vdy) {
            if x0 + dx < 0 || x0 + dx > 7 || y0 + dy < 0 || y0 + dy > 7 {
                continue;
            }
            let p = Position::new(x0 + dx, y0 + dy);
            let target_ref = self.board[p];
            if target_ref.active() && target_ref.color() != knight.color() {
                buffer[count] = Move::new_capture(pref, target_ref, MoveFlags::EMPTY);
                count += 1;
            } else if !target_ref.active() {
                buffer[count] = Move::new_move(pref, knight.position, p, MoveFlags::EMPTY);
                count += 1;
            }
        }

        count
    }

    pub fn insert_pawn_moves(&self, pref: PieceRef, pawn: &Piece, buffer: &mut [Move]) -> usize {
        let mut count = 0;
        let p = pawn.position;

        if pawn.color() == Color::White {
            if p.y() < 6 {
                if !self.board[p.dp(0, 1)].active() {
                    buffer[count] = Move::new_move(pref, p, p.dp(0, 1), MoveFlags::EMPTY);
                    count += 1;

                    if p.y() == 1 && !self.board[p.dp(0, 2)].active() {
                        buffer[count] = Move::new_move(pref, p, p.dp(0, 2), MoveFlags::EMPTY);
                        count += 1;
                    }
                }

                if p.x() > 0 {
                    let target_ref = self.board[p.dp(-1, 1)];
                    if target_ref.active() && target_ref.color() == Color::Black {
                        buffer[count] = Move::new_capture(pref, target_ref, MoveFlags::EMPTY);
                        count += 1;
                    }
                }

                if p.x() < 7 {
                    let target_ref = self.board[p.dp(1, 1)];
                    if target_ref.active() && target_ref.color() == Color::Black {
                        buffer[count] = Move::new_capture(pref, target_ref, MoveFlags::EMPTY);
                        count += 1;
                    }
                }
            } else if p.y() == 6 {
                // promotions
                if !self.board[p.dp(0, 1)].active() {
                    buffer[count] = Move::new_promote(pref, Type::Queen, p.dp(0, 1));
                    count += 1;
                }

                if p.x() > 0 {
                    let target_ref = self.board[p.dp(-1, 1)];
                    if target_ref.active() && target_ref.color() == Color::Black {
                        buffer[count] = Move::new_capture_and_promote(pref, target_ref, Type::Queen);
                        count += 1;
                    }
                }

                if p.x() < 7 {
                    let target_ref = self.board[p.dp(1, 1)];
                    if target_ref.active() && target_ref.color() == Color::Black {
                        buffer[count] = Move::new_capture_and_promote(pref, target_ref, Type::Queen);
                        count += 1;
                    }
                }
            }
        } else {
            if p.y() > 1 {
                if !self.board[p.dp(0, -1)].active() {
                    buffer[count] = Move::new_move(pref, p, p.dp(0, -1), MoveFlags::EMPTY);
                    count += 1;

                    if p.y() == 6 && !self.board[p.dp(0, -2)].active() {
                        buffer[count] = Move::new_move(pref, p, p.dp(0, -2), MoveFlags::EMPTY);
                        count += 1;
                    }
                }

                if p.x() > 0 {
                    let target_ref = self.board[p.dp(-1, -1)];
                    if target_ref.active() && target_ref.color() == Color::White {
                        buffer[count] = Move::new_capture(pref, target_ref, MoveFlags::EMPTY);
                        count += 1;
                    }
                }

                if p.x() < 7 {
                    let target_ref = self.board[p.dp(1, -1)];
                    if target_ref.active() && target_ref.color() == Color::White {
                        buffer[count] = Move::new_capture(pref, target_ref, MoveFlags::EMPTY);
                        count += 1;
                    }
                }
            } else if p.y() == 1 {
                // promotions
                if !self.board[p.dp(0, -1)].active() {
                    buffer[count] = Move::new_promote(pref, Type::Queen, p.dp(0, -1));
                    count += 1;
                }

                if p.x() > 0 {
                    let target_ref = self.board[p.dp(-1, -1)];
                    if target_ref.active() && target_ref.color() == Color::White {
                        buffer[count] = Move::new_capture_and_promote(pref, target_ref, Type::Queen);
                        count += 1;
                    }
                }

                if p.x() < 7 {
                    let target_ref = self.board[p.dp(1, -1)];
                    if target_ref.active() && target_ref.color() == Color::White {
                        buffer[count] = Move::new_capture_and_promote(pref, target_ref, Type::Queen);
                        count += 1;
                    }
                }
            }
        }

        count
    }


    pub fn insert_rook_moves(&self, pref: PieceRef, rook: &Piece, board_flags: &MoveFlags, buffer: &mut [Move]) -> usize {
        let mut count = 0;
        let (x0, y0) = (rook.position.x(), rook.position.y());

        let mut flags = MoveFlags::EMPTY;

        // these positions are the same for white AND black
        if !board_flags.contains(MoveFlags::QUEEN_ROOK_MOVED) && x0 == 0 {
            flags.set(MoveFlags::QUEEN_ROOK_MOVED, true);
        } else if !board_flags.contains(MoveFlags::KING_ROOK_MOVED) && x0 == 7 {
            flags.set(MoveFlags::KING_ROOK_MOVED, true);
        }

        // left
        for x in (0..x0).rev() {
            let p = Position::new(x, y0);
            let target_ref = self.board[p];
            if target_ref.active() {
                if target_ref.color() != rook.color() {
                    buffer[count] = Move::new_capture(pref, target_ref, flags);
                    count += 1;
                }
                break;
            } else {
                buffer[count] = Move::new_move(pref, rook.position, p, flags);
                count += 1;
            }
        }

        // right
        for x in x0 + 1..8 {
            let p = Position::new(x, y0);
            let target_ref = self.board[p];
            if target_ref.active() {
                if target_ref.color() != rook.color() {
                    buffer[count] = Move::new_capture(pref, target_ref, flags);
                    count += 1;
                }
                break;
            } else {
                buffer[count] = Move::new_move(pref, rook.position, p, flags);
                count += 1;
            }
        }

        // up
        for y in y0 + 1..8 {
            let p = Position::new(x0, y);
            let target_ref = self.board[p];
            if target_ref.active() {
                if target_ref.color() != rook.color() {
                    buffer[count] = Move::new_capture(pref, target_ref, flags);
                    count += 1;
                }
                break;
            } else {
                buffer[count] = Move::new_move(pref, rook.position, p, flags);
                count += 1;
            }
        }

        // down
        for y in (0..y0).rev() {
            let p = Position::new(x0, y);
            let target_ref = self.board[p];
            if target_ref.active() {
                if target_ref.color() != rook.color() {
                    buffer[count] = Move::new_capture(pref, target_ref, flags);
                    count += 1;
                }
                break;
            } else {
                buffer[count] = Move::new_move(pref, rook.position, p, flags);
                count += 1;
            }
        }


        count
    }

    pub fn insert_bishop_moves(&self, pref: PieceRef, bishop: &Piece, buffer: &mut [Move]) -> usize {
        use std::cmp::min;
        let mut count = 0;
        let (x0, y0) = (bishop.position.x(), bishop.position.y());

        // top right
        for n in 1..min(7 - x0, 7 - y0) {
            let p = Position::new(x0 + n, y0 + n);
            let target_ref = self.board[p];
            if target_ref.active() {
                if target_ref.color() != bishop.color() {
                    buffer[count] = Move::new_capture(pref, target_ref, MoveFlags::EMPTY);
                    count += 1;
                }
                break;
            } else {
                buffer[count] = Move::new_move(pref, bishop.position, p, MoveFlags::EMPTY);
                count += 1;
            }
        }

        // top left
        for n in 1..min(x0, 7 - y0) {
            let p = Position::new(x0 - n, y0 + n);
            let target_ref = self.board[p];
            if target_ref.active() {
                if target_ref.color() != bishop.color() {
                    buffer[count] = Move::new_capture(pref, target_ref, MoveFlags::EMPTY);
                    count += 1;
                }
                break;
            } else {
                buffer[count] = Move::new_move(pref, bishop.position, p, MoveFlags::EMPTY);
                count += 1;
            }
        }

        // bottom left
        for n in 1..min(x0, y0) {
            let p = Position::new(x0 - n, y0 - n);
            let target_ref = self.board[p];
            if target_ref.active() {
                if target_ref.color() != bishop.color() {
                    buffer[count] = Move::new_capture(pref, target_ref, MoveFlags::EMPTY);
                    count += 1;
                }
                break;
            } else {
                buffer[count] = Move::new_move(pref, bishop.position, p, MoveFlags::EMPTY);
                count += 1;
            }
        }


        // bottom right
        for n in 1..min(7 - x0, y0) {
            let p = Position::new(x0 + n, y0 - n);
            let target_ref = self.board[p];
            if target_ref.active() {
                if target_ref.color() != bishop.color() {
                    buffer[count] = Move::new_capture(pref, target_ref, MoveFlags::EMPTY);
                    count += 1;
                }
                break;
            } else {
                buffer[count] = Move::new_move(pref, bishop.position, p, MoveFlags::EMPTY);
                count += 1;
            }
        }


        count
    }

    pub fn insert_queen_moves(&self, pref: PieceRef, queen: &Piece, buffer: &mut [Move]) -> usize {
        // TODO: Clean this up, we pretend everything moved already for the queen, with MoveFlags::FULL
        let mut count = self.insert_rook_moves(pref, queen, &MoveFlags::FULL, buffer);
        count += self.insert_bishop_moves(pref, queen, &mut buffer[count..]);

        count
    }

    pub fn make_move(&mut self, m: &Move) {
        match m.action {
            Action::Move { start, end } => {
                debug_assert!(self.piece_at(end).is_none());
                debug_assert_eq!(self.piece_at(start).unwrap().position, start);
                self.board[end] = m.piece_ref;
                self.board[start] = PieceRef::null();
                self.pieces[m.piece_ref].position = end;
                let flags = if m.piece_ref.color() == Color::White { &mut self.white_move_flags } else { &mut self.black_move_flags };
                *flags |= m.flags;
            }
            Action::Capture { target } => {
                // copy positions
                let start = self.pieces[m.piece_ref].position;
                let end = self.pieces[target].position;

                // swap positions
                self.pieces[m.piece_ref].position = end;
                self.pieces[target].position = start;
                self.pieces[target].set_active(false);

                // update board
                self.board[end] = m.piece_ref;
                self.board[start] = PieceRef::null();

                let flags = if m.piece_ref.color() == Color::White { &mut self.white_move_flags } else { &mut self.black_move_flags };
                *flags |= m.flags;
            }
            Action::Promote { t, end } => {
                let start = self.pieces[m.piece_ref].position;

                // move the piece
                self.board[end] = m.piece_ref;
                self.board[start] = PieceRef::null();

                self.pieces[m.piece_ref].position = end;
                // change the type
                self.pieces[m.piece_ref].set_type(t);
            }
            Action::CaptureAndPromote { target, t } => {
                // copy positions
                let start = self.pieces[m.piece_ref].position;
                let end = self.pieces[target].position;

                // swap positions
                self.pieces[m.piece_ref].position = end;
                self.pieces[target].position = start;
                self.pieces[target].set_active(false);

                // update board
                self.board[end] = m.piece_ref;
                self.board[start] = PieceRef::null();

                // change the type
                self.pieces[m.piece_ref].set_type(t);
            }
            Action::None => unreachable!(),
            Action::Castle => {
                if m.piece_ref.color() == Color::White {
                    if m.flags.contains(MoveFlags::QUEEN_ROOK_MOVED) {
                        self.move_piece((0, 0), (3, 0));
                        self.move_piece((4, 0), (2, 0));
                    } else if m.flags.contains(MoveFlags::KING_ROOK_MOVED) {
                        self.move_piece((7, 0), (5, 0));
                        self.move_piece((4, 0), (6, 0));
                    } else {
                        // it makes no sense to castle without a single rook moving
                        unreachable!();
                    }
                    self.white_move_flags |= m.flags;
                } else {
                    if m.flags.contains(MoveFlags::QUEEN_ROOK_MOVED) {
                        self.move_piece((0, 7), (3, 7));
                        self.move_piece((4, 7), (2, 7));
                    } else if m.flags.contains(MoveFlags::KING_ROOK_MOVED) {
                        self.move_piece((7, 7), (5, 7));
                        self.move_piece((4, 7), (6, 7));
                    } else {
                        // it makes no sense to castle without a single rook moving
                        unreachable!();
                    }
                    self.black_move_flags |= m.flags;
                }
            }
            Action::Evaluate => unreachable!()
        }
    }

    pub fn revert_move(&mut self, m: &Move) {
        match m.action {
            Action::Move { start, end } => {
                debug_assert!(self.piece_at(start).is_none());
                debug_assert_eq!(self.piece_at(end).unwrap().position, end);
                self.board[start] = m.piece_ref;
                self.board[end] = PieceRef::null();
                self.pieces[m.piece_ref].position = start;

                let flags = if m.piece_ref.color() == Color::White { &mut self.white_move_flags } else { &mut self.black_move_flags };
                *flags ^= m.flags;
            }
            Action::Capture { target } => {
                let end = self.pieces[m.piece_ref].position;
                let start = self.pieces[target].position;

                // swap back positions
                self.pieces[m.piece_ref].position = start;
                self.pieces[target].position = end;
                self.pieces[target].set_active(true);

                // update board
                self.board[start] = m.piece_ref;
                self.board[end] = target;

                let flags = if m.piece_ref.color() == Color::White { &mut self.white_move_flags } else { &mut self.black_move_flags };
                *flags ^= m.flags;
            }
            Action::Promote { end, .. } => {
                let start = if m.piece_ref.color() == Color::White {
                    end.dp(0, -1)
                } else {
                    end.dp(0, 1)
                };

                self.board[start] = m.piece_ref;
                self.board[end] = PieceRef::null();

                self.pieces[m.piece_ref].position = start;
                self.pieces[m.piece_ref].set_type(Type::Pawn);
            }
            Action::CaptureAndPromote { target, .. } => {
                let end = self.pieces[m.piece_ref].position;
                let start = self.pieces[target].position;

                // swap back positions
                self.pieces[m.piece_ref].position = start;
                self.pieces[target].position = end;
                self.pieces[target].set_active(true);

                // update board
                self.board[start] = m.piece_ref;
                self.board[end] = target;

                // change back the type
                self.pieces[m.piece_ref].set_type(Type::Pawn);
            }
            Action::None => unreachable!(),
            Action::Castle => {
                if m.piece_ref.color() == Color::White {
                    if m.flags.contains(MoveFlags::QUEEN_ROOK_MOVED) {
                        self.move_piece((3, 0), (0, 0));
                        self.move_piece((2, 0), (4, 0));
                    } else if m.flags.contains(MoveFlags::KING_ROOK_MOVED) {
                        self.move_piece((5, 0), (7, 0));
                        self.move_piece((6, 0), (4, 0));
                    } else {
                        // it makes no sense to castle without a single rook moving
                        unreachable!();
                    }
                    self.white_move_flags ^= m.flags;
                } else {
                    if m.flags.contains(MoveFlags::QUEEN_ROOK_MOVED) {
                        self.move_piece((3, 7), (0, 7));
                        self.move_piece((2, 7), (4, 7));
                    } else if m.flags.contains(MoveFlags::KING_ROOK_MOVED) {
                        self.move_piece((5, 7), (7, 7));
                        self.move_piece((6, 7), (4, 7));
                    } else {
                        // it makes no sense to castle without a single rook moving
                        unreachable!();
                    }
                    self.black_move_flags ^= m.flags;
                }
            }
            Action::Evaluate => unreachable!()
        }
    }
}

// Formatting

impl Debug for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let line = "  +---+---+---+---+---+---+---+---+";
        writeln!(f, "{}", line);
        for y in 0..8 {
            write!(f, "{} |", 8 - y);
            for x in 0..8 {
                let p = Position::new(x, 7 - y);
                let maybe_piece = self.piece_at(p);
                if let Some(piece) = maybe_piece {
                    write!(f, " {} |", piece.char());
                } else {
                    write!(f, "   |");
                }
            }
            writeln!(f, "\n{}", line);
        }
        writeln!(f, "    a   b   c   d   e   f   g   h");
        Ok(())
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
    use super::*;
    use super::utils::*;

    #[test]
    fn piece_list_add_piece() {
        let mut piece_list = PieceList::new();
        let piece = Piece::new(Color::Black, Type::Rook, Position::new(1, 1));
        let pref = piece_list.add_new_piece(piece);
        assert_eq!(pref.color(), piece.color());
        assert_eq!(piece_list[pref], piece);
    }

    #[test]
    fn add_black_piece() {
        use super::{Piece, Position, Color, Type, Board};
        let mut board = Board::new();
        let p = Position::new(1, 1);
        board.add_new_piece(Color::Black, Type::Rook, p.x(), p.y());

        let piece_ref = board.board[p];
        assert!(piece_ref.active());
        assert_eq!(piece_ref.color(), Color::Black);

        assert!(board.piece_at(p).is_some());

        let piece = board.piece_at(p).unwrap();
        assert_eq!(piece.color(), Color::Black);
        assert_eq!(piece.t(), Type::Rook);
        assert_eq!(piece.position, p);
    }

    #[test]
    fn white_castle() {
        let mut board = Board::new();
        let mut initial_buffer = vec![Move::none(); 1_000];
        board.add_new_piece(Color::White, Type::King, 4, 0);
        board.add_new_piece(Color::White, Type::Rook, 0, 0);
        board.add_new_piece(Color::White, Type::Rook, 7, 0);

        let count = board.insert_all_moves(Color::White, &mut initial_buffer[0..]);
        assert_eq!(count, 5 + 3 + 2 + 7 + 7 + 2);

        let moves: Vec<Move> = initial_buffer[0..count].iter().filter_map(|m| match m.action() {
            Action::Castle => Some(*m),
            _ => None
        }).collect();

        assert_eq!(moves.len(), 2);

        let m1 = moves.iter().find(|m| m.flags().contains(MoveFlags::QUEEN_ROOK_MOVED)).unwrap();
        let m2 = moves.iter().find(|m| m.flags().contains(MoveFlags::KING_ROOK_MOVED)).unwrap();


        board.make_move(m1);
        assert_eq!(board.piece_at_xy(0, 0), None);
        assert_eq!(board.piece_at_xy(4, 0), None);
        assert!(board.piece_at_xy(2, 0).is_some());
        assert!(board.piece_at_xy(3, 0).is_some());
        assert_eq!(board.white_move_flags, MoveFlags::KING_MOVED | MoveFlags::QUEEN_ROOK_MOVED);


        board.revert_move(m1);
        assert!(board.piece_at_xy(0, 0).is_some());
        assert!(board.piece_at_xy(4, 0).is_some());
        assert!(board.piece_at_xy(2, 0).is_none());
        assert!(board.piece_at_xy(3, 0).is_none());
        assert_eq!(board.white_move_flags, MoveFlags::EMPTY);

        board.make_move(m2);
        assert_eq!(board.piece_at_xy(7, 0), None);
        assert_eq!(board.piece_at_xy(4, 0), None);
        assert!(board.piece_at_xy(6, 0).is_some());
        assert!(board.piece_at_xy(5, 0).is_some());
        assert_eq!(board.white_move_flags, MoveFlags::KING_MOVED | MoveFlags::KING_ROOK_MOVED);

        board.revert_move(m2);
        assert!(board.piece_at_xy(7, 0).is_some());
        assert!(board.piece_at_xy(4, 0).is_some());
        assert!(board.piece_at_xy(6, 0).is_none());
        assert!(board.piece_at_xy(5, 0).is_none());
        assert_eq!(board.white_move_flags, MoveFlags::EMPTY);
    }
}