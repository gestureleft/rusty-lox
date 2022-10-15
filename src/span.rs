use std::cmp::{max, min};

#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Span {
        Self { start, end }
    }

    pub fn slice<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }

    pub fn combine(self, other: Span) -> Span {
        Self::new(min(self.start, other.start), max(self.end, other.end))
    }
}
