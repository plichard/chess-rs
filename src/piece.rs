use crate::utils::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

pub type PieceIndex = u8;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Piece {
    pub t: Type,
    pub position: Position,
    pub index: PieceIndex,
    pub color: Color,
}

impl Piece {
    pub fn character(&self) -> &str {
        use Color::*;
        use Type::*;
        match (self.color, self.t) {
            (White, Pawn) => "P",
            (Black, Pawn) => "p",
            (White, Rook) => "R",
            (Black, Rook) => "r",
            (White, King) => "K",
            (Black, King) => "k",
            (White, Bishop) => "B",
            (Black, Bishop) => "b",
            (White, Queen) => "Q",
            (Black, Queen) => "q",
            (White, Knight) => "N",
            (Black, Knight) => "n",
        }
    }

    pub fn moved(&self, position: Position) -> Self {
        Self {
            t: self.t,
            position,
            index: self.index,
            color: self.color,
        }
    }

    pub fn base_value(&self) -> i16 {
        match self.t {
            Type::Pawn => 100,
            Type::Bishop => 300,
            Type::Knight => 300,
            Type::Rook => 500,
            Type::Queen => 900,
            Type::King => 10000,
        }
    }

    pub fn value(&self) -> i16 {
        let pos_value = match self.t {
            Type::Pawn => {
                // let v = match self.color {
                //     Color::White => (10*self.position.y as i32),
                //     Color::Black => 10*(7-self.position.y as i32)
                // };
                // v
                0
                //50*v/(1+v)
            }
            Type::Rook => {
                // priority in the center
                let dx = i8::min(self.position.x, 7 - self.position.x) as i16;
                let dy = i8::min(self.position.y, 7 - self.position.y) as i16;
                ((dx + dy) * 5) as i16
            }
            Type::King => {
                // priority in the center
                let dx = i8::min(self.position.x, 7 - self.position.x) as i16;
                let dy = i8::min(self.position.y, 7 - self.position.y) as i16;
                ((dx + dy) * 1) as i16
            }
            Type::Bishop => 0,
            Type::Queen => {
                // priority in the center
                let dx = i8::min(self.position.x, 7 - self.position.x) as i16;
                let dy = i8::min(self.position.y, 7 - self.position.y) as i16;
                ((dx + dy) * 5) as i16
            }
            Type::Knight => {
                // priority in the center
                let dx = i8::min(self.position.x, 7 - self.position.x) as i16;
                let dy = i8::min(self.position.y, 7 - self.position.y) as i16;
                ((dx + dy) * 5) as i16
            }
        };

        self.base_value() + pos_value
    }
}
