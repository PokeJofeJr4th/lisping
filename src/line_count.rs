struct LineCount<I: Iterator<Item = char>> {
    iter: I,
    row: usize,
    col: usize,
}

impl<I: Iterator<Item = char>> LineCount<I> {
    pub const fn new(iter: I) -> Self {
        Self {
            iter,
            row: 0,
            col: 0,
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for LineCount<I> {
    type Item = (usize, usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.iter.next()?;
        if c == '\n' {
            self.row += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        Some((self.row, self.col, c))
    }
}

pub trait LineCountable {
    fn line_count(self) -> impl Iterator<Item = (usize, usize, char)>;
}

impl<I: Iterator<Item = char>> LineCountable for I {
    fn line_count(self) -> impl Iterator<Item = (usize, usize, char)> {
        LineCount::new(self)
    }
}
