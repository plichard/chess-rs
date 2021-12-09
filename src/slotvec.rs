

#[derive(Copy, Clone)]
pub struct StaticSlotVec<T, const N: usize> {
    data: [T; N],
    n: usize,

}

impl<T, const N: usize> StaticSlotVec<T, N> where T: Copy + PartialEq {
    pub fn new(default_value: T) -> Self {
        Self {
            data: [default_value; N],
            n: 0
        }
    }

    pub fn insert(&mut self, value: T) {
        self.data[self.n] = value;
        self.n += 1;
    }

    pub fn remove(&mut self, value: T) {
        if let Some(index) = self.data.iter().position(|&x| x == value) {
            self.data[index] = self.data[self.n - 1];
            self.n -= 1;
        }
    }

    pub fn len(&self) -> usize {
        self.n
    }

    pub fn clear(&mut self) {
        self.n = 0;
    }
}