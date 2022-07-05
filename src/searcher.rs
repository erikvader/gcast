mod lcs;
mod util;

use delegate::delegate;
use std::cell::{Ref, RefCell};
use std::ops::Range;

use self::lcs::{dp::Element,LCS};
use self::util::compact_to_ranges;

#[derive(Debug)]
pub struct TaggedLCS {
    lcs: LCS,
    index: usize,
    best_indices: RefCell<Option<Vec<usize>>>,
}

impl TaggedLCS {
    fn new(string: String, index: usize) -> Self {
        Self {
            lcs: LCS::new(string),
            index,
            best_indices: RefCell::new(None),
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    fn clear_best_spread(&self) {
        self.best_indices.replace(None);
    }

    fn calc_best_indices(&self) -> Ref<Vec<usize>> {
        if !self.has_best_indices() {
            self.best_indices
                .replace(Some(self.lcs.get_leftmost_indices()));
        }

        // TODO: really want to spend 10x more time to get the best one?
        // if self.best_indices.borrow().is_none() {
        //     let mut iter = self.lcs.get_all_paths().take(Self::MAX_PATHS);

        //     let first: Option<Vec<_>> =
        //         iter.next().map(|slice| path_to_indices(slice).collect());

        //     if let Some(first_path) = first {
        //         let first_spread = self.lcs.spread_ref(&first_path);
        //         let (min_path, _) = iter.fold(
        //             (first_path, first_spread),
        //             |org @ (_, min_spread), new| {
        //                 let new_spread = self.lcs.spread(path_to_indices(new));
        // TODO: also compare first position to get the leftmost one
        //                 if new_spread < min_spread {
        //                     (path_to_indices(new).collect(), new_spread)
        //                 } else {
        //                     org
        //                 }
        //             },
        //         );

        //         self.best_indices.replace(Some(min_path));
        //     } else {
        //         self.best_indices.replace(Some(Vec::new()));
        //     }
        // }
        let opt: Ref<Option<_>> = self.best_indices.borrow();
        Ref::map(opt, |x| x.as_ref().expect("should have been calculated"))
    }

    fn has_best_indices(&self) -> bool {
        self.best_indices.borrow().is_some()
    }

    delegate! {
        to self.lcs {
            pub fn length(&self) -> usize;
            fn grid_len(&self) -> usize;
            pub fn get_interspersed<T1, T2, ON, OFF>(
                &self, indices: &[usize], on_lcs: ON, off_lcs: OFF
            ) -> String
                where T1: std::fmt::Display,
                      T2: std::fmt::Display,
                      ON: Fn(char) -> T1,
                      OFF: Fn(char) -> T2;
        }
    }

    fn push(&mut self, c: char) -> Result<(), ()> {
        self.clear_best_spread();
        self.lcs.push(c)
    }
    fn pop(&mut self) {
        self.clear_best_spread();
        self.lcs.pop()
    }

    pub fn spread(&self) -> usize {
        self.lcs.spread_ref(self.calc_best_indices().iter())
    }
    pub fn first_pos(&self) -> Option<usize> {
        self.lcs.first_pos(self.calc_best_indices().iter())
    }
    pub fn get_best_indices(&self) -> Vec<usize> {
        self.calc_best_indices().to_vec()
    }
    pub fn get_best_indices_compact(&self) -> Vec<Range<usize>> {
        compact_to_ranges(self.calc_best_indices().iter().copied(), 1)
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

#[test]
fn test_lcs_tagged_lcs() {
    let mut lcs = TaggedLCS::new("asd".into(), 5);
    assert!(lcs.push('s').is_ok());
    assert!(!lcs.has_best_indices());
    assert_eq!(lcs.get_best_indices(), vec![1]);
    assert!(lcs.has_best_indices());

    assert!(lcs.push('d').is_ok());
    assert!(!lcs.has_best_indices());
    assert_eq!(lcs.get_best_indices(), vec![1, 2]);
    assert!(lcs.has_best_indices());
}

// searcher ///////////////////////////////////////////////////////////////////
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

    pub fn get_sorted(&mut self) -> impl Iterator<Item = &TaggedLCS> {
        self.active.sort_unstable();
        self.active.iter()
    }

    pub fn get_sorted_take(&mut self, len: usize) -> impl Iterator<Item = &TaggedLCS> {
        if len > 0 {
            if len <= self.active.len() {
                let (beg, _, _) = self.active.select_nth_unstable(len - 1);
                beg.sort_unstable();
            } else {
                self.active.sort_unstable();
            }
        }
        self.active.iter().take(len)
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
}

#[cfg(test)]
mod test {
    use super::*;

    impl Searcher {
        fn assert_invariants(&self) {
            assert_eq!(self.num_chars, self.search.chars().count());
            assert!(self
                .active
                .iter()
                .all(|x| x.lcs.grid_columns() == self.num_chars));
            assert!(self.active.iter().all(|x| x.lcs.length() == self.num_chars));
            assert!(!self
                .inactive
                .iter()
                .any(|x| x.lcs.grid_columns() > self.num_chars));
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
}
