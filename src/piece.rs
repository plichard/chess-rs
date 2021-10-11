use crate::utils::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Color {
    White,
    Black
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Type {
    Pawn,
    // Bishop,
    // Knight,
    Rook,
    // Queen,
    King
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
            (White, Pawn) => 'P',
            (Black, Pawn) => 'p',
            (White, Rook) => 'R',
            (Black, Rook) => 'r',
            (White, King) => 'K',
            (Black, King) => 'k',
        }
    }

    pub fn moved(&self, position: Position) -> Self {
        Self {
            t: self.t,
            position,
            color: self.color,
            index: self.index
        }
    }

    pub fn base_value(&self) -> i64 {
        match self.t {
            Type::Pawn => 1000,
            Type::Rook => 5000,
            Type::King => 100000,
        }
    }

    pub fn value(&self) -> i64 {
        let pos_value = match self.t {
            Type::Pawn => {
                let v = match self.color {
                    Color::White => (10*self.position.y as i64),
                    Color::Black => 10*(7-self.position.y as i64)
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
            Type::King => 0,
        };

        self.base_value() + pos_value
    }
}
