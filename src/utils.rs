#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Position {
    pub x: i8,
    pub y: i8,
}

impl Position {
    pub fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }


    pub fn up(&self, n: i8) -> Option<Self> {
        if self.y < 7 - n + 1 {
            Some(Position::new(self.x, self.y + n))
        } else {
            None
        }
    }

    pub fn down(&self, n: i8) -> Option<Self> {
        if self.y >= n {
            Some(Position::new(self.x, self.y - n))
        } else {
            None
        }
    }

    pub fn left(&self, n: i8) -> Option<Self> {
        if self.x >= n {
            Some(Position::new(self.x - n, self.y))
        } else {
            None
        }
    }

    pub fn right(&self, n: i8) -> Option<Self> {
        if self.x < 7 - n + 1 {
            Some(Position::new(self.x + n, self.y))
        } else {
            None
        }
    }

    pub fn up_left(&self, n: i8) -> Option<Self> {
        if self.y < 7 - n + 1 && self.x >= n {
            Some(Position::new(self.x - n, self.y + n))
        } else {
            None
        }
    }

    pub fn down_left(&self, n: i8) -> Option<Self> {
        if self.y >= n && self.x >= n {
            Some(Position::new(self.x - n, self.y - n))
        } else {
            None
        }
    }

    pub fn up_right(&self, n: i8) -> Option<Self> {
        if self.y < 7 - n + 1 && self.x < 7 - n + 1 {
            Some(Position::new(self.x + n, self.y + n))
        } else {
            None
        }
    }

    pub fn down_right(&self, n: i8) -> Option<Self> {
        if self.y >= n && self.x < 7 - n + 1 {
            Some(Position::new(self.x + n, self.y - n))
        } else {
            None
        }
    }
}

