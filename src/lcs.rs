use delegate::delegate;
use itertools::Itertools;
use std::{cmp::max, collections::HashSet, num::NonZeroUsize, ops::Range};
use streaming_iterator::StreamingIterator;

trait Recurrence: FnMut(Element, Element, Element) -> Result<Element, ()> {}
impl<T> Recurrence for T where T: FnMut(Element, Element, Element) -> Result<Element, ()> {}

// Element ////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Element {
    length: u8,
    matched: bool,
}

impl Element {
    #[cfg(not(test))]
    const MAX: u8 = u8::MAX;
    #[cfg(test)]
    const MAX: u8 = 10;

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

    fn new_matched(other: Self) -> Result<Self, ()> {
        if other.length() < Self::MAX {
            Ok(Self {
                length: other.length() + 1,
                matched: true,
            })
        } else {
            Err(())
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

    fn generate_col(&mut self, mut recur: impl Recurrence) -> Result<(), ()> {
        let org_len = self.grid.len();
        self.grid.reserve(self.height.get());
        let new_col = self.columns_non_zero() + 1;

        let mut up = Element::empty();
        for row in 1..=self.height.get() {
            let left = self.get(row, new_col - 1).expect("left should exist");
            let upleft = self
                .get(row - 1, new_col - 1)
                .expect("up left should exist");

            if let Ok(x) = recur(left, up, upleft) {
                self.grid.push(x);
                up = x;
            } else {
                self.grid.truncate(org_len);
                return Err(());
            }
        }
        Ok(())
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
    assert!(g.generate_col(|_a, _b, _c| Ok(ele)).is_ok());
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
    assert!(g.generate_col(|_a, _b, _c| Ok(ele)).is_ok());
    assert!(g.generate_col(|_a, _b, _c| Ok(ele)).is_ok());
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
    // TODO: quotes, reserved chars
    // TODO: quote togglar bara
    // TODO: searchern håller koll på denna
    // TODO: LCS har bool på sina pushes som bestämmer ifall den innan ska ha matchat för
    // att den nya ska få matcha. LCS ska inte spara om den är i quite-läge eller inte.
    const QUOTE: char = '\'';

    pub fn new(compare: String) -> Self {
        let len = compare.chars().count();
        assert!(len > 0, "can't do LCS on empty string");
        LCS {
            compare,
            dp: Grid::new_usize(len),
        }
    }

    pub fn push(&mut self, c: char) -> Result<(), ()> {
        // TODO: hantera QUOTE och UNQUOTE, eller ha två metoder som Searchern ansvarar
        // att kalla på? och en assert här att c inte är QUOTE?

        let mut cmp = self.compare.chars();
        self.dp.generate_col(|left, up, upleft| {
            let cur = cmp
                .next()
                .expect("compare and grid height should be the same length");

            // TODO: ha quote-logik som säger att denna matchar endast om den snett
            // ovanför också matchade
            if c == cur {
                Element::new_matched(upleft)
            } else {
                Ok(Element::new_not_matched(left, up))
            }
        })
    }

    pub fn push_str(&mut self, s: &str) -> Result<(), usize> {
        let mut count = 0;
        for c in s.chars() {
            if self.push(c).is_ok() {
                count += 1;
            } else {
                return Err(count);
            }
        }
        Ok(())
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

    fn get_all_paths(&self) -> LCSIterator<'_> {
        LCSIterator::new(&self)
    }

    pub fn get_all_indices(&self) -> impl Iterator<Item = Vec<usize>> + '_ {
        self.get_all_paths()
            .map_deref(|path| path_to_indices(path).collect())
    }

    pub fn get_some_indices(&self) -> Vec<usize> {
        self.get_all_paths()
            .next()
            .map(|path| path_to_indices(path).collect())
            .unwrap_or_else(|| Vec::new())
    }

    pub fn get_string<'a>(&self, indices: impl IntoIterator<Item = &'a usize>) -> String {
        let chars: Vec<char> = self.compare.chars().collect();
        indices.into_iter().map(|i| chars[*i]).collect()
    }

    pub fn get_interspersed<T1, T2, ON, OFF>(
        &self,
        indices: &[usize],
        on_lcs: ON,
        off_lcs: OFF,
    ) -> String
    where
        T1: std::fmt::Display,
        T2: std::fmt::Display,
        ON: Fn(char) -> T1,
        OFF: Fn(char) -> T2,
    {
        let mut res = String::new();
        for (i, c) in self.compare.chars().enumerate() {
            if indices.binary_search(&i).is_ok() {
                res.push_str(&on_lcs(c).to_string());
            } else {
                res.push_str(&off_lcs(c).to_string());
            }
        }
        res
    }

    pub fn spread<'a>(&self, indices: impl IntoIterator<Item = &'a usize>) -> usize {
        indices
            .into_iter()
            .tuple_windows()
            .map(|(a, b)| b - a - 1)
            .sum()
    }

    pub fn first_pos<'a>(
        &self,
        indices: impl IntoIterator<Item = &'a usize>,
    ) -> Option<usize> {
        indices.into_iter().next().copied()
    }

    fn grid_len(&self) -> usize {
        self.dp.len()
    }
}

#[test]
fn test_lcs() {
    let mut lcs = LCS::new("GAC".into());
    assert!(lcs.push_str("AGCAT").is_ok());
    assert_eq!(lcs.length(), 2);
    let indices = lcs.get_some_indices();
    assert!(vec!["AC", "GC", "GA"].contains(&lcs.get_string(&indices).as_str()));
}

#[test]
fn test_empty_lcs() {
    let lcs = LCS::new("asd".into());
    let indices = lcs.get_some_indices();
    assert!(indices.is_empty());
    assert!(lcs.get_string(&indices).is_empty());
}

#[test]
fn test_lcs_intersperse() {
    let mut lcs = LCS::new("asd".into());
    assert!(lcs.push('s').is_ok());
    let indices = lcs.get_some_indices();
    assert_eq!(
        lcs.get_interspersed(&indices, |c| format!("1{}2", c), |c| c),
        "a1s2d"
    );
}

#[test]
fn test_lcs_tightness() {
    let mut lcs = LCS::new("src/lc".into());
    assert!(lcs.push_str("src").is_ok());
    assert_eq!(lcs.length(), 3);

    let all: Vec<Vec<usize>> = lcs.get_all_indices().sorted().dedup().collect();
    assert_eq!(all, vec![vec![0, 1, 2], vec![0, 1, 5]]);
    assert!(all.contains(&lcs.get_some_indices()));

    let strings: Vec<String> =
        all.iter().map(|indices| lcs.get_string(indices)).collect();
    assert_eq!(strings, vec!["src", "src"]);

    let first_poses: Vec<Option<usize>> =
        all.iter().map(|indices| lcs.first_pos(indices)).collect();
    assert_eq!(first_poses, vec![Some(0), Some(0)]);

    let spreads: Vec<usize> = all.iter().map(|indices| lcs.spread(indices)).collect();
    assert_eq!(spreads, vec![0, 3]);
}

#[test]
fn test_lcs_pop() {
    let mut lcs = LCS::new("asd".into());
    assert!(lcs.push('s').is_ok());
    assert!(lcs.push('d').is_ok());
    let indices = lcs.get_some_indices();
    assert_eq!(lcs.get_string(&indices), "sd");

    lcs.pop();
    let indices = lcs.get_some_indices();
    assert_eq!(lcs.get_string(&indices), "s");
}

// LCS iterator ///////////////////////////////////////////////////////////////
struct LCSIterator<'a> {
    lcs: &'a LCS,
    path: Vec<(usize, usize)>,
    dfs: Vec<(usize, usize)>,
}

impl<'a> LCSIterator<'a> {
    fn new(lcs: &'a LCS) -> Self {
        LCSIterator {
            lcs,
            path: Vec::new(),
            dfs: if lcs.length() > 0 {
                let start = lcs.dp.bottom_right();
                vec![start]
            } else {
                Vec::new()
            },
        }
    }
}

impl<'a> StreamingIterator for LCSIterator<'a> {
    type Item = [(usize, usize)];

    fn advance(&mut self) {
        while let Some(curpos @ (row, col)) = self.dfs.pop() {
            if let Some(&last) = self.path.last() {
                if last == curpos {
                    self.path.pop();
                    continue;
                }
            }

            if row == 0 || col == 0 {
                self.path.push(curpos);
                self.dfs.push(curpos);
                return;
            }

            let cur = self.lcs.dp.get(row, col).expect("should be in bounds");
            let left = self.lcs.dp.get(row, col - 1).expect("should be in bounds");
            let up = self.lcs.dp.get(row - 1, col).expect("should be in bounds");

            self.path.push(curpos);
            self.dfs.push(curpos);
            if cur.length() == up.length() {
                self.dfs.push((row - 1, col));
            }
            if cur.length() == left.length() {
                self.dfs.push((row, col - 1));
            }
            if cur.matched() {
                self.dfs.push((row - 1, col - 1));
            }
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.path.is_empty() {
            assert!(self.dfs.is_empty());
            None
        } else {
            Some(&self.path)
        }
    }
}

fn is_upleft(from: (usize, usize), to: (usize, usize)) -> bool {
    from.0 == to.0 + 1 && from.1 == to.1 + 1
}

fn path_to_indices(path: &[(usize, usize)]) -> impl Iterator<Item = usize> + '_ {
    path.windows(2)
        .filter(|x| is_upleft(x[0], x[1]))
        .map(|x| x[0].0 - 1)
        .rev()
}

#[test]
fn test_lcs_iterator() {
    let mut lcs = LCS::new("GAC".into());
    assert!(lcs.push_str("AGCAT").is_ok());
    assert_eq!(lcs.length(), 2);

    let mut paths: Vec<Vec<(usize, usize)>> = LCSIterator::new(&lcs).owned().collect();
    paths.sort_unstable();
    let org_len = paths.len();
    paths.dedup();
    assert_eq!(org_len, paths.len(), "should not have duplicates");

    let indices: Vec<Vec<usize>> = paths
        .iter()
        .map(|x| path_to_indices(x).collect())
        .dedup()
        .collect();

    let strings: Vec<String> = indices
        .iter()
        .map(|indices| lcs.get_string(indices))
        .sorted()
        .collect();

    assert_eq!(strings, vec!["AC", "GA", "GC"]);
}

#[test]
fn test_lcs_iterator_empty() {
    {
        let lcs = LCS::new("asd".into());
        assert_eq!(lcs.length(), 0);
        let mut iter = LCSIterator::new(&lcs);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    {
        let mut lcs = LCS::new("asd".into());
        assert!(lcs.push('x').is_ok());
        assert_eq!(lcs.length(), 0);
        let mut iter = LCSIterator::new(&lcs);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }
}

// searcher ///////////////////////////////////////////////////////////////////
#[derive(Debug)]
struct TaggedLCS {
    lcs: LCS,
    index: usize,
    // TODO: en cell som memoizar subsekvens med lägsta spread
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
            fn push(&mut self, c: char) -> Result<(), ()>;
            fn pop(&mut self);
            fn grid_len(&self) -> usize;
        }
    }

    // TODO:
    fn spread(&self) -> usize {
        0
    }
    fn first_pos(&self) -> Option<usize> {
        Some(0)
    }

    // TODO: spread-funktion som hittar den med lägst spread
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
    pub fn new<T, It>(candidates: It) -> Self
    where
        T: Into<String>,
        It: IntoIterator<Item = T>,
    {
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

    pub fn push_str(&mut self, s: &str) -> Result<(), usize> {
        let mut count = 0;
        for c in s.chars() {
            if self.push(c).is_ok() {
                count += 1;
            } else {
                return Err(count);
            }
        }
        Ok(())
    }

    pub fn push(&mut self, c: char) -> Result<(), ()> {
        self.search.push(c);
        self.num_chars += 1;

        for i in 0..self.active.len() {
            if self.active[i].push(c).is_err() {
                for j in 0..i {
                    self.active[j].pop();
                }
                self.num_chars -= 1;
                self.search.pop();
                return Err(());
            }
        }

        self.active
            .sort_unstable_by_key(|lcs| lcs.length() != self.num_chars);
        while let Some(last) = self.active.last() {
            if last.length() == self.num_chars {
                break;
            }
            self.inactive.push(self.active.pop().expect("last != None"));
        }
        Ok(())
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

    pub fn get_sorted_take(&mut self, len: usize) -> impl Iterator<Item = &LCS> {
        if len > 0 {
            if len <= self.active.len() {
                let (beg, _, _) = self.active.select_nth_unstable(len - 1);
                beg.sort_unstable();
            } else {
                self.active.sort_unstable();
            }
        }
        self.active.iter().map(|lcs| &lcs.lcs).take(len)
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

    assert!(searcher.push('a').is_ok());
    searcher.assert_invariants();
    assert_eq!(searcher.get_sorted().count(), 4);
    assert_eq!(searcher.get_search(), "a");

    searcher.pop();
    searcher.assert_invariants();
    assert_eq!(searcher.get_sorted().count(), 5);
    assert_eq!(searcher.get_search(), "");
    assert_eq!(searcher.size_indication(), 0, "all grids should be empty");

    assert!(searcher.push_str("aab").is_ok());
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

#[test]
fn test_lcs_searcher_too_long() {
    let s = "aaaaaaaaaaa";
    assert!(s.chars().count() > Element::MAX.into());
    let mut searcher = Searcher::new(vec![s]);
    assert_eq!(Err(10), searcher.push_str(s));
    assert_eq!(&s[..10], searcher.get_search());
    assert_eq!(10, searcher.num_chars);
}

#[test]
fn test_lcs_searcher_empty() {
    let mut searcher = Searcher::new::<String, Vec<String>>(Vec::new());
    assert!(searcher.push('a').is_ok());
    searcher.assert_invariants();
    assert_eq!(searcher.get_sorted().count(), 0);
    assert_eq!(searcher.get_search(), "a");
}

#[test]
fn test_lcs_searcher_longer() {
    let mut searcher = Searcher::new(vec!["ab"]);
    assert_eq!(searcher.push_str("abb"), Ok(()));
    searcher.assert_invariants();
    assert_eq!(searcher.get_search(), "abb");
    assert_eq!(searcher.get_sorted().count(), 0);
    assert_eq!(searcher.get_sorted_take(10).count(), 0);

    searcher.pop();
    searcher.assert_invariants();
    assert_eq!(searcher.get_search(), "ab");
    assert_eq!(searcher.get_sorted().count(), 1);
}

// util ///////////////////////////////////////////////////////////////////////
fn compact_to_ranges<T, It>(verbose: It, one: T) -> Vec<Range<T>>
where
    T: PartialOrd<T> + Copy + std::ops::Add<Output = T>,
    It: IntoIterator<Item = T>,
{
    assert!(one + one > one, "unexpected behaviour of 'one'");
    let mut iter = verbose.into_iter();
    if let Some(first) = iter.next() {
        let mut res = vec![Range {
            start: first,
            end: first + one,
        }];

        iter.for_each(|x| {
            let end = &mut res.last_mut().unwrap().end;
            assert!(x >= *end, "must be strictly increasing");
            if *end == x {
                *end = x + one;
            } else {
                res.push(Range {
                    start: x,
                    end: x + one,
                });
            }
        });
        res
    } else {
        Vec::new()
    }
}

#[test]
fn test_compact_ranges() {
    assert_eq!(compact_to_ranges(vec![], 1), vec![]);
    assert_eq!(compact_to_ranges(vec![1, 2, 3], 1), vec![1..4]);
    assert_eq!(compact_to_ranges(vec![1, 3, 4], 1), vec![1..2, 3..5]);
    assert_eq!(
        compact_to_ranges(vec![1, 3, 7, 9], 1),
        vec![1..2, 3..4, 7..8, 9..10]
    );
}

#[test]
#[should_panic(expected = "must be strictly increasing")]
fn test_compact_invalid_input() {
    compact_to_ranges(vec![1, 1], 1);
}
