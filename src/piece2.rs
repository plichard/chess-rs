use std::fmt::{Debug, Display, Formatter};
use sfml::audio::listener::position;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Type {
    Pawn = 0,
    Rook = 1,
    Knight = 2,
    Bishop = 3,
    Queen = 4,
    King = 5,
    BadValue1 = 6,
    BadValue2 = 7,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Color {
    White = 0,
    Black = 1,
}


#[derive(Copy, Clone, PartialOrd, PartialEq, Eq)]
pub struct Position {
    index: u8,
}

impl Position {
    pub fn zero() -> Self {
        Self {
            index: 0
        }
    }

    pub fn index(&self) -> usize {
        self.index as usize
    }

    pub fn new(x: i8, y: i8) -> Self {
        Self {
            index: (x + y * 8) as u8
        }
    }

    pub fn x(&self) -> i8 {
        (self.index & 0b111) as i8
    }

    pub fn y(&self) -> i8 {
        ((self.index & 0b111000) >> 3) as i8
    }
}

#[derive(Copy, Clone)]
pub struct Piece {
    data: u8,
    // 3 bits type, 1 bit color, 1 bit active
    pub position: Position,
}

impl Piece {
    pub fn t(&self) -> Type {
        unsafe {
            // This is safe because all the 3-bit u8
            // values are represented in the enum
            std::mem::transmute(self.data & 0b111)
        }
    }

    pub fn color(&self) -> Color {
        unsafe {
            // This is safe because the bitmask can only produce 0 or 1
            std::mem::transmute((self.data >> 3) & 0b1)
        }
    }

    pub fn new(color: Color, t: Type, position: Position) -> Self {
        let data = ((color as u8) << 3) + (t as u8);

        Self {
            data,
            position,
        }
    }

    pub fn null() -> Self {
        Self {
            data: 0,
            position: Position::zero(),
        }
    }

    pub fn active(&self) -> bool {
        (self.data & 0b10000) != 0
    }

    pub fn set_active(&mut self, active: bool) {
        if active {
            self.data |= 0b00010000;
        } else {
            self.data &= 0b11101111;
        }
    }
}

#[derive(Copy, Clone)]
pub struct PieceRef {
    index: u8,
}

impl PieceRef {
    pub fn index(&self) -> usize {
        (self.index & 0b11111) as usize
    }

    pub fn color(&self) -> Color {
        unsafe {
            // This is safe because the bitmask can only produce 0 or 1
            std::mem::transmute((self.index >> 4) & 0b1)
        }
    }

    pub fn active(&self) -> bool {
        (self.index & 0b100000) != 0
    }

    pub fn null() -> Self {
        Self {
            index: 0
        }
    }

    pub fn new(color: Color, index: usize) -> Self {
        Self {
            index: (index as u8) + ((color as u8) << 4) + (0b1 << 5)
        }
    }

    pub fn set_active(&mut self, active: bool) {
        if active {
            self.index |= 0b00100000;
        } else {
            self.index &= 0b11011111;
        }
    }
}


// Formatting functions

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Color::White => "White",
            Color::Black => "Black"
        })
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Type::Pawn => "Pawn",
            Type::Rook => "Rook",
            Type::Knight => "Knight",
            Type::Bishop => "Bishop",
            Type::Queen => "Queen",
            Type::King => "King",
            Type::BadValue1 => "BadValue1",
            Type::BadValue2 => "BadValue2",
        })
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", ('a' as u8 + self.x() as u8) as char, 1 + self.y())
    }
}

impl Debug for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", ('a' as u8 + self.x() as u8) as char, 1 + self.y())
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} ({})", self.color(), self.t(), self.position)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn create_piece1() {
        use super::{Piece, Position, Color, Type};
        let rook1 = Piece::new(Color::White, Type::Rook, Position::new(3, 4));
        assert_eq!(rook1.t(), Type::Rook);
        assert_eq!(rook1.color(), Color::White);
        assert_eq!(format!("{}", rook1.position), "d5");
    }

    #[test]
    fn type_size() {
        use super::{Piece, PieceRef, Position};
        assert_eq!(std::mem::size_of::<Piece>(), 2);
        assert_eq!(std::mem::size_of::<PieceRef>(), 1);
        assert_eq!(std::mem::size_of::<Position>(), 1);
    }
}