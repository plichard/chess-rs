use crate::utils::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Color {
    White,
    Black
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Type {
    Pawn,
    Bishop,
    // Knight,
    Rook,
    Queen,
    King
}

pub type PieceIndex = u8;

#[derive(Debug, Copy, Clone)]
pub struct Piece {
    pub t: Type,
    pub position: Position,
    pub index: PieceIndex,
    pub color: Color
}

impl Piece {
    pub fn character(&self) -> &str {
        use Color::*;
        use Type::*;
        match (self.color, self.t) {
            (White, Pawn) => "P",
            (Black, Pawn) => "︎p",
            (White, Rook) => "R",
            (Black, Rook) => "r",
            (White, King) => "K",
            (Black, King) => "k",
            (White, Bishop) => "B",
            (Black, Bishop) => "b",
            (White, Queen) => "Q",
            (Black, Queen) => "q",
        }
    }

    pub fn moved(&self, position: Position) -> Self {
        Self {
            t: self.t,
            position,
            index: self.index,
            color: self.color
        }
    }

    pub fn base_value(&self) -> i32 {
        match self.t {
            Type::Pawn => 1000,
            Type::Bishop => 3000,
            Type::Rook => 5000,
            Type::Queen => 9000,
            Type::King => 100000,
        }
    }

    pub fn value(&self) -> i32 {
        let pos_value = match self.t {
            Type::Pawn => {
                let v = match self.color {
                    Color::White => (10*self.position.y as i32),
                    Color::Black => 10*(7-self.position.y as i32)
                };
                v
                //50*v/(1+v)
            },
            Type::Rook => {
                let v = match self.color {
                    Color::White => {
                        if self.position.y == 6 {
                            100
                        } else {
                            0
                        }
                    },
                    Color::Black => {
                        if self.position.y == 1 {
                            100
                        } else {
                            0
                        }
                    }
                };
                v
            },
            Type::King => {
                let v = match self.color {
                    Color::White => (10*self.position.y as i32),
                    Color::Black => 10*(7-self.position.y as i32)
                };
                v
            },
            Type::Bishop => {
                0
            },
            Type::Queen => 0,
        };

        self.base_value() + pos_value
    }
}
