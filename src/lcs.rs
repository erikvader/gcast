use std::{cmp::max, num::NonZeroUsize};

trait Recurrence: FnMut(Element, Element, Element) -> Element {}
impl<T> Recurrence for T where T: FnMut(Element, Element, Element) -> Element {}

// Element ////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Element {
    length: u8,
    matched: bool,
}

impl Element {
    const MAX: u8 = u8::MAX;

    fn empty() -> Self {
        Self {
            length: 0,
            matched: false,
        }
    }

    fn matched(self) -> bool {
        self.matched
    }

    fn length(self) -> u8 {
        self.length
    }

    fn new_matched(other: Self) -> Self {
        Self {
            length: other.length() + 1,
            matched: true,
        }
    }

    fn new_not_matched(other1: Self, other2: Self) -> Self {
        Self {
            length: max(other1.length(), other2.length()),
            matched: false,
        }
    }
}

// Grid ///////////////////////////////////////////////////////////////////////

struct Grid {
    grid: Vec<Element>,
    height: NonZeroUsize,
}

impl Grid {
    fn new_usize(height: usize) -> Self {
        Grid::new(NonZeroUsize::new(height).expect("can't have Grid with height 0"))
    }

    fn new(height: NonZeroUsize) -> Self {
        Grid {
            grid: Vec::new(),
            height,
        }
    }

    fn is_empty(&self) -> bool {
        self.grid.is_empty()
    }

    fn columns_non_zero(&self) -> usize {
        self.grid.len() / self.height
    }

    fn bottom_right(&self) -> (usize, usize) {
        (self.height.get(), self.columns_non_zero())
    }

    fn get(&self, row: usize, col: usize) -> Option<Element> {
        if row == 0 || col == 0 {
            return Some(Element::empty());
        }
        if (row - 1) >= self.height.get() || (col - 1) >= self.columns_non_zero() {
            return None;
        }
        self.grid
            .get((col - 1) * self.height.get() + (row - 1))
            .copied()
    }

    fn get_last(&self) -> Option<Element> {
        self.grid.last().copied()
    }

    fn pop_col(&mut self) {
        for _ in 0..self.height.get() {
            self.grid.pop().expect("grid should not be empty");
        }
    }

    fn generate_col(&mut self, mut recur: impl Recurrence) {
        self.grid.reserve(self.height.get());
        let new_col = self.columns_non_zero() + 1;

        let mut up = Element::empty();
        for row in 1..=self.height.get() {
            let left = self.get(row, new_col - 1).expect("left should exist");
            let upleft = self
                .get(row - 1, new_col - 1)
                .expect("up left should exist");

            let x = recur(left, up, upleft);
            self.grid.push(x);
            up = x;
        }
    }
}

#[test]
fn test_empty() {
    let g = Grid::new_usize(3);
    assert_eq!(g.columns_non_zero(), 0);
    assert!(g.is_empty());
    assert_eq!(g.get_last(), None);
    assert_eq!(g.get(0, 0), Some(Element::empty()));
    assert_eq!(g.get(1, 0), Some(Element::empty()));
    assert_eq!(g.get(0, 1), Some(Element::empty()));
    assert_eq!(g.get(1, 1), None);
    assert_eq!(g.get(42, 67), None);
}

#[test]
fn test_one_column() {
    let mut g = Grid::new_usize(3);
    let ele = Element {
        length: 42,
        matched: false,
    };
    g.generate_col(|_a, _b, _c| ele);
    assert_eq!(g.grid.len(), 3);
    assert_eq!(g.columns_non_zero(), 1);
    assert_eq!(g.get(4, 1), None);
    assert_eq!(g.get(3, 1), Some(ele));
    assert_eq!(g.get(1, 1), Some(ele));
    assert_eq!(g.get(1, 2), None);
    assert_eq!(g.get_last(), g.get(3, 1));
}

#[test]
fn test_pop_column() {
    let mut g = Grid::new_usize(3);
    let ele = Element {
        length: 42,
        matched: false,
    };
    g.generate_col(|_a, _b, _c| ele);
    g.generate_col(|_a, _b, _c| ele);
    assert_eq!(g.grid.len(), 6);
    assert_eq!(g.columns_non_zero(), 2);
    assert_eq!(g.get(4, 2), None);
    assert_eq!(g.get(3, 2), Some(ele));
    assert_eq!(g.get(1, 2), Some(ele));
    assert_eq!(g.get(1, 3), None);

    g.pop_col();
    assert_eq!(g.grid.len(), 3);
    assert_eq!(g.columns_non_zero(), 1);
}

// LCS ////////////////////////////////////////////////////////////////////////

pub struct LCS {
    compare: String,
    dp: Grid,
}

impl LCS {
    pub fn new(compare: String) -> Self {
        let len = compare.chars().count();
        assert!(len > 0, "can't do LCS on empty string");
        LCS {
            compare,
            dp: Grid::new_usize(len),
        }
    }

    pub fn push(&mut self, c: char) {
        let mut cmp = self.compare.chars();
        self.dp.generate_col(|left, up, upleft| {
            let cur = cmp
                .next()
                .expect("compare and grid height should be the same length");

            if c == cur {
                Element::new_matched(upleft)
            } else {
                Element::new_not_matched(left, up)
            }
        });
    }

    pub fn push_str(&mut self, s: &str) {
        s.chars().for_each(|c| self.push(c));
    }

    pub fn pop(&mut self) {
        self.dp.pop_col();
    }

    pub fn is_empty(&self) -> bool {
        self.dp.is_empty()
    }

    pub fn get_compare(&self) -> &str {
        self.compare.as_ref()
    }

    pub fn length(&self) -> usize {
        self.dp.get_last().map_or(0, |x| x.length().into())
    }

    pub fn get_indices(&self) -> Vec<usize> {
        let mut lcs: Vec<usize> = Vec::new();
        let (mut row, mut col) = self.dp.bottom_right();

        while row > 0 && col > 0 {
            let ele = self.dp.get(row, col).expect("should be in bounds");
            let left = self.dp.get(row, col - 1).expect("should be in bounds");
            let up = self.dp.get(row - 1, col).expect("should be in bounds");

            if ele.matched() {
                lcs.push(row - 1);
                row -= 1;
                col -= 1;
            } else if left.length() > up.length() {
                col -= 1;
            } else {
                row -= 1;
            }
        }

        lcs.reverse();
        lcs
    }

    pub fn get_string(&self) -> String {
        let chars: Vec<char> = self.compare.chars().collect();
        self.get_indices().into_iter().map(|i| chars[i]).collect()
    }

    pub fn get_grouped(&self) -> Vec<&str> {
        let mut groups: Vec<&str> = Vec::new();
        groups.push("");
        todo!();
        groups
    }
}

#[test]
fn test_lcs() {
    let mut lcs = LCS::new("GAC".into());
    lcs.push_str("AGCAT");
    assert_eq!(lcs.length(), 2);
    assert!(vec!["AC", "GC", "GA"].contains(&lcs.get_string().as_str()));
}

// searcher ///////////////////////////////////////////////////////////////////
pub struct Searcher {
    cands: Vec<LCS>,
    search: String,
}

impl Searcher {
    pub fn new(candidates: Vec<String>) -> Self {
        Self {
            cands: candidates.into_iter().map(|c| LCS::new(c)).collect(),
            search: String::new(),
        }
    }

    fn sort(&mut self) {
        self.cands
            .sort_by_key(|lcs| usize::max_value() - lcs.length());
    }

    pub fn push_str(&mut self, s: &str) {
        self.search.push_str(s);
        self.cands.iter_mut().for_each(|lcs| lcs.push_str(s));
        self.sort();
    }

    pub fn push(&mut self, c: char) {
        self.search.push(c);
        self.cands.iter_mut().for_each(|lcs| lcs.push(c));
        self.sort();
    }

    pub fn pop(&mut self) {
        self.search.pop();
        self.cands.iter_mut().for_each(|lcs| lcs.pop());
        self.sort();
    }

    pub fn get_sorted(&self) -> &[LCS] {
        &self.cands
    }

    pub fn get_search(&self) -> &str {
        self.search.as_str()
    }
}
