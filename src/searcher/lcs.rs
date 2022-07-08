use itertools::Itertools;
use streaming_iterator::StreamingIterator;

use crate::searcher::{
    matcher::{first_pos_ref, spread, spread_ref},
    util::get_interspersed,
};

use self::{
    dp::{Element, Grid},
    iterator::{path_to_indices, LCSIterator},
};

use super::matcher::Matcher;

pub mod dp;
pub mod iterator;

#[derive(Debug)]
pub struct LCS<T> {
    compare: T,
    dp: Grid,
}

impl<T> LCS<T>
where
    T: AsRef<str>,
{
    pub fn new(compare: T) -> Self {
        let len = compare.as_ref().chars().count();
        assert!(len > 0, "can't do LCS on empty string");
        LCS {
            compare,
            dp: Grid::new_usize(len),
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

    pub fn is_empty(&self) -> bool {
        self.dp.is_empty()
    }

    pub fn longest_seq_str<'a>(
        &self,
        indices: impl IntoIterator<Item = &'a usize>,
    ) -> String {
        let chars: Vec<char> = self.get_compare().chars().collect();
        indices.into_iter().map(|i| chars[*i]).collect()
    }

    pub fn length(&self) -> usize {
        self.dp.get_last().map_or(0, |x| x.length().into())
    }

    fn get_all_paths(&self) -> LCSIterator<'_, T> {
        LCSIterator::new(&self)
    }

    pub fn get_all_indices(&self) -> impl Iterator<Item = Vec<usize>> + '_ {
        self.get_all_paths()
            .map_deref(|path| path_to_indices(path).collect())
    }

    pub fn get_leftmost_indices(&self) -> Vec<usize> {
        self.get_all_paths()
            .next()
            .map(|path| path_to_indices(path).collect())
            .unwrap_or_else(|| Vec::new())
    }

    pub fn grid_len(&self) -> usize {
        self.dp.len()
    }

    pub fn grid_columns(&self) -> usize {
        self.dp.columns_non_zero()
    }
}

impl<T> Matcher for LCS<T>
where
    T: AsRef<str>,
{
    fn push(&mut self, c: char) -> Result<(), ()> {
        let mut cmp = self.compare.as_ref().chars();
        self.dp.generate_col(|left, up, upleft| {
            let cur = cmp
                .next()
                .expect("compare and grid height should be the same length");

            if c.eq_ignore_ascii_case(&cur) {
                Element::new_matched(upleft)
            } else {
                Ok(Element::new_not_matched(left, up))
            }
        })
    }

    fn pop(&mut self) {
        self.dp.pop_col();
    }

    fn get_indices(&self) -> Vec<usize> {
        self.get_leftmost_indices()
    }

    fn get_compare(&self) -> &str {
        self.compare.as_ref()
    }
}

#[test]
fn test_lcs() {
    let mut lcs = LCS::new("GAC");
    assert!(lcs.push_str("AGCAT").is_ok());
    assert_eq!(lcs.length(), 2);
    let indices = lcs.get_leftmost_indices();
    assert!(vec!["AC", "GC", "GA"].contains(&lcs.longest_seq_str(&indices).as_str()));
}

#[test]
fn test_empty_lcs() {
    let lcs = LCS::new("asd");
    let indices = lcs.get_leftmost_indices();
    assert!(indices.is_empty());
    assert!(lcs.longest_seq_str(&indices).is_empty());
}

#[test]
fn test_lcs_intersperse() {
    let mut lcs = LCS::new("asd");
    assert!(lcs.push('s').is_ok());
    let indices = lcs.get_leftmost_indices();
    assert_eq!(
        get_interspersed(lcs.get_compare(), &indices, |c| format!("1{}2", c), |c| c),
        "a1s2d"
    );
}

#[test]
fn test_lcs_all_subsequences() {
    let mut lcs = LCS::new("src/lc");
    assert!(lcs.push_str("src").is_ok());
    assert_eq!(lcs.length(), 3);

    let all: Vec<Vec<usize>> = lcs.get_all_indices().sorted().dedup().collect();
    assert_eq!(all, vec![vec![0, 1, 2], vec![0, 1, 5]]);
    assert!(all.contains(&lcs.get_leftmost_indices()));

    let strings: Vec<String> = all
        .iter()
        .map(|indices| lcs.longest_seq_str(indices))
        .collect();
    assert_eq!(strings, vec!["src", "src"]);

    let first_poses: Vec<Option<usize>> =
        all.iter().map(|indices| first_pos_ref(indices)).collect();
    assert_eq!(first_poses, vec![Some(0), Some(0)]);

    let spreads: Vec<usize> = all.iter().map(|indices| spread_ref(indices)).collect();
    assert_eq!(spreads, vec![0, 3]);
}

#[test]
fn test_lcs_leftmost() {
    let mut lcs = LCS::new("scsrrcc/src");
    assert!(lcs.push_str("src").is_ok());
    assert_eq!(lcs.length(), 3);

    let left = lcs.get_leftmost_indices();
    assert_eq!(left, vec![0, 3, 5]);
    assert_eq!(spread(left), 3);
}

#[test]
fn test_lcs_pop() {
    let mut lcs = LCS::new("asd");
    assert!(lcs.push('s').is_ok());
    assert!(lcs.push('d').is_ok());
    let indices = lcs.get_leftmost_indices();
    assert_eq!(lcs.longest_seq_str(&indices), "sd");

    lcs.pop();
    let indices = lcs.get_leftmost_indices();
    assert_eq!(lcs.longest_seq_str(&indices), "s");
}
