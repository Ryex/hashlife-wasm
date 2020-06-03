// retangle

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Rectangle {
    width: usize,
    height: usize,
}

impl Rectangle {
    pub fn new(width: usize, height: usize) -> Self {
        Rectangle { width, height }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}
