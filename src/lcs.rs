use delegate::delegate;
use std::{cmp::max, num::NonZeroUsize, ops::Range};

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
#[derive(Debug)]
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

    fn len(&self) -> usize {
        self.grid.len()
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
    assert_eq!(g.len(), 3);
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
    assert_eq!(g.len(), 6);
    assert_eq!(g.columns_non_zero(), 2);
    assert_eq!(g.get(4, 2), None);
    assert_eq!(g.get(3, 2), Some(ele));
    assert_eq!(g.get(1, 2), Some(ele));
    assert_eq!(g.get(1, 3), None);

    g.pop_col();
    assert_eq!(g.len(), 3);
    assert_eq!(g.columns_non_zero(), 1);
}

// LCS ////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
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
            let cur = self.dp.get(row, col).expect("should be in bounds");
            let left = self.dp.get(row, col - 1).expect("should be in bounds");
            let up = self.dp.get(row - 1, col).expect("should be in bounds");

            if cur.length() == up.length() {
                row -= 1;
            } else if cur.length() == left.length() {
                col -= 1;
            } else {
                assert!(
                    cur.matched(),
                    "cur must be matched here, else the algorithm is not correct"
                );
                lcs.push(row - 1);
                row -= 1;
                col -= 1;
            }
        }

        lcs.reverse();
        lcs
    }

    pub fn get_ranges(&self) -> Vec<Range<usize>> {
        // TODO:
        todo!("ge tillbaka en kompaktare representation av get_indices")
    }

    pub fn get_string(&self) -> String {
        let chars: Vec<char> = self.compare.chars().collect();
        self.get_indices().into_iter().map(|i| chars[i]).collect()
    }

    pub fn get_interspersed<T1, T2, ON, OFF>(&self, on_lcs: ON, off_lcs: OFF) -> String
    where
        T1: std::fmt::Display,
        T2: std::fmt::Display,
        ON: Fn(char) -> T1,
        OFF: Fn(char) -> T2,
    {
        let mut res = String::new();
        let lcs = self.get_indices();
        for (i, c) in self.compare.chars().enumerate() {
            if lcs.binary_search(&i).is_ok() {
                res.push_str(&on_lcs(c).to_string());
            } else {
                res.push_str(&off_lcs(c).to_string());
            }
        }
        res
    }

    pub fn spread(&self) -> usize {
        let ind = self.get_indices();
        if ind.len() <= 1 {
            return 0;
        }
        ind.windows(2).map(|x| x[1] - x[0] - 1).sum()
    }

    pub fn first_pos(&self) -> Option<usize> {
        self.get_indices().first().copied()
    }

    fn grid_len(&self) -> usize {
        self.dp.len()
    }
}

#[test]
fn test_lcs() {
    let mut lcs = LCS::new("GAC".into());
    lcs.push_str("AGCAT");
    assert_eq!(lcs.length(), 2);
    assert!(vec!["AC", "GC", "GA"].contains(&lcs.get_string().as_str()));
}

#[test]
fn test_empty_lcs() {
    let lcs = LCS::new("asd".into());
    assert!(lcs.get_indices().is_empty());
    assert!(lcs.get_string().is_empty());
}

#[test]
fn test_lcs_intersperse() {
    let mut lcs = LCS::new("asd".into());
    lcs.push('s');
    assert_eq!(lcs.get_interspersed(|c| format!("1{}2", c), |c| c), "a1s2d");
}

#[test]
fn test_lcs_tightness() {
    let mut lcs = LCS::new("src/lc".into());
    lcs.push_str("src");
    assert_eq!(lcs.length(), 3);
    assert_eq!(lcs.get_string(), "src");
    assert_eq!(lcs.first_pos(), Some(0));
    assert_eq!(lcs.spread(), 0);
}

#[test]
fn test_lcs_pop() {
    let mut lcs = LCS::new("asd".into());
    lcs.push('s');
    lcs.push('d');
    assert_eq!(lcs.get_string(), "sd");

    lcs.pop();
    assert_eq!(lcs.get_string(), "s");
}

// searcher ///////////////////////////////////////////////////////////////////
#[derive(Debug)]
struct TaggedLCS {
    lcs: LCS,
    index: usize,
}

impl TaggedLCS {
    fn new(string: String, index: usize) -> Self {
        Self {
            lcs: LCS::new(string),
            index,
        }
    }

    delegate! {
        to self.lcs {
            fn length(&self) -> usize;
            fn spread(&self) -> usize;
            fn first_pos(&self) -> Option<usize>;
            fn push(&mut self, c: char);
            fn pop(&mut self);
            fn grid_len(&self) -> usize;
        }
    }
}

impl Ord for TaggedLCS {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = self;
        let b = other;
        b.length()
            .cmp(&a.length())
            .then_with(|| a.spread().cmp(&b.spread()))
            .then_with(|| a.first_pos().cmp(&b.first_pos()))
            .then_with(|| a.index.cmp(&b.index))
    }
}

impl PartialOrd for TaggedLCS {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TaggedLCS {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for TaggedLCS {}

#[derive(Debug)]
pub struct Searcher {
    active: Vec<TaggedLCS>,
    inactive: Vec<TaggedLCS>,
    search: String,
    num_chars: usize,
}

impl Searcher {
    pub fn new<T: Into<String>>(candidates: Vec<T>) -> Self {
        Self {
            active: candidates
                .into_iter()
                .enumerate()
                .map(|(i, c)| TaggedLCS::new(c.into(), i))
                .collect(),
            inactive: Vec::new(),
            search: String::new(),
            num_chars: 0,
        }
    }

    pub fn push_str(&mut self, s: &str) {
        s.chars().for_each(|c| self.push(c));
    }

    pub fn push(&mut self, c: char) {
        self.search.push(c);
        self.active.iter_mut().for_each(|lcs| lcs.push(c));
        self.num_chars += 1;

        self.active
            .sort_unstable_by_key(|lcs| lcs.length() != self.num_chars);
        while let Some(last) = self.active.last() {
            if last.length() == self.num_chars {
                break;
            }
            self.inactive.push(self.active.pop().expect("last != None"));
        }
    }

    pub fn pop(&mut self) {
        if self.num_chars == 0 {
            return;
        }
        self.search.pop();
        self.active.iter_mut().for_each(|lcs| lcs.pop());
        self.num_chars -= 1;

        while let Some(last) = self.inactive.last() {
            if last.length() != self.num_chars {
                break;
            }
            let mut popped = self.inactive.pop().expect("last != None");
            popped.pop();
            self.active.push(popped);
        }
    }

    pub fn get_sorted(&mut self) -> impl Iterator<Item = &LCS> {
        self.active.sort_unstable();
        self.active.iter().map(|lcs| &lcs.lcs)
    }

    pub fn get_search(&self) -> &str {
        self.search.as_str()
    }

    pub fn size_indication(&self) -> usize {
        self.active
            .iter()
            .chain(self.inactive.iter())
            .map(|lcs| lcs.grid_len())
            .sum::<usize>()
            * std::mem::size_of::<Element>()
    }

    #[cfg(test)]
    fn assert_invariants(&self) {
        assert_eq!(self.num_chars, self.search.chars().count());
        assert!(self
            .active
            .iter()
            .all(|x| x.lcs.dp.columns_non_zero() == self.num_chars));
        assert!(self.active.iter().all(|x| x.lcs.length() == self.num_chars));
        assert!(!self
            .inactive
            .iter()
            .any(|x| x.lcs.dp.columns_non_zero() > self.num_chars));
        assert!(!self
            .inactive
            .iter()
            .any(|x| x.lcs.length() >= self.num_chars));
    }
}

#[test]
fn test_lcs_searcher() {
    let mut searcher = Searcher::new(vec!["aaa", "aab", "aa", "abab", "bbbb"]);
    assert_eq!(searcher.get_sorted().count(), 5);
    assert_eq!(searcher.size_indication(), 0, "all grids should be empty");
    assert_eq!(searcher.get_search(), "");
    searcher.assert_invariants();

    searcher.push('a');
    searcher.assert_invariants();
    assert_eq!(searcher.get_sorted().count(), 4);
    assert_eq!(searcher.get_search(), "a");

    searcher.pop();
    searcher.assert_invariants();
    assert_eq!(searcher.get_sorted().count(), 5);
    assert_eq!(searcher.get_search(), "");
    assert_eq!(searcher.size_indication(), 0, "all grids should be empty");

    searcher.push_str("aab");
    searcher.assert_invariants();
    assert_eq!(searcher.get_sorted().count(), 2);

    searcher.pop();
    searcher.assert_invariants();
    assert_eq!(searcher.get_sorted().count(), 4);

    searcher.pop();
    searcher.assert_invariants();
    assert_eq!(searcher.get_sorted().count(), 4);

    searcher.pop();
    searcher.assert_invariants();
    assert_eq!(searcher.get_sorted().count(), 5);
    assert_eq!(searcher.get_search(), "");
    assert_eq!(searcher.size_indication(), 0, "all grids should be empty");
}
