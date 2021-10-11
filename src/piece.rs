use crate::utils::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Color {
    White,
    Black
}

#[derive(Copy, Clone, Debug)]
pub enum Type {
    Pawn,
    // Bishop,
    // Knight,
    // Rook,
    // Queen,
    // King
}

impl Type {
    pub fn value(&self) -> i64 {
        match self {
            Type::Pawn => 1,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Type::Pawn => "Pawn"
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Piece {
    pub t: Type,
    pub position: Position,
    pub color: Color,
    pub index: u8
}

impl Piece {
    pub fn character(&self) -> char {
        use Color::*;
        use Type::*;
        match (self.color, self.t) {
            (White, Pawn) => '\u{2659}',
            (Black, Pawn) => '\u{265F}'
        }
    }
}
