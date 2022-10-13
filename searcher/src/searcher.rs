use std::collections::HashSet;

use rayon::prelude::*;
use regex::Regex;

use self::{
    compile::{compile_search_term_to_regexes, Result},
    r#match::Match,
};

mod compile;
mod r#match;
pub mod util;

pub use compile::CompileError;

#[derive(Debug)]
pub struct SearchRes<'a> {
    mat: Match,
    index: usize,
    string: &'a str,
}

impl Ord for SearchRes<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = self;
        let b = other;
        a.mat
            .spread()
            .cmp(&b.mat.spread())
            .then_with(|| a.mat.first().cmp(&b.mat.first()))
            .then_with(|| a.index.cmp(&b.index))
    }
}

impl PartialOrd for SearchRes<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SearchRes<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for SearchRes<'_> {}

impl SearchRes<'_> {
    pub fn get_string(&self) -> &str {
        self.string
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_match(&self) -> &Match {
        &self.mat
    }
}

impl<'a> SearchRes<'a> {
    fn from_bytes(bytes: &HashSet<usize>, index: usize, string: &'a str) -> Self {
        SearchRes {
            index,
            string,
            mat: Match::from_vec(
                string
                    .char_indices()
                    .map(|(b, _)| b)
                    .enumerate()
                    .filter(|(_, b)| bytes.contains(b))
                    .map(|(i, _)| i)
                    .collect(),
            ),
        }
    }

    fn empty(index: usize, string: &'a str) -> Self {
        SearchRes {
            index,
            string,
            mat: Match::empty(),
        }
    }

    fn new(index: usize, string: &'a str, indices: Vec<usize>) -> Self {
        SearchRes {
            index,
            string,
            mat: Match::from_vec(indices),
        }
    }
}

fn run_regexes_get_bytes(regs: &Vec<Regex>, string: &str) -> Option<HashSet<usize>> {
    assert!(!regs.is_empty());
    let mut bytes = HashSet::new();
    for reg in regs {
        if let Some(caps) = reg.captures(string) {
            caps.iter()
                .skip(1)
                .flat_map(|mat| mat.expect("all capture groups should exist").range())
                .for_each(|s| {
                    bytes.insert(s);
                });
        } else {
            return None;
        }
    }
    Some(bytes)
}

fn search_with_regex<'a, It, I>(regs: &Vec<Regex>, candidates: It) -> Vec<SearchRes<'a>>
where
    I: AsRef<str> + 'a,
    It: IntoParallelIterator<Item = &'a I>,
    <It as IntoParallelIterator>::Iter: IndexedParallelIterator,
{
    candidates
        .into_par_iter()
        .enumerate()
        .filter_map(|(i, cand)| {
            run_regexes_get_bytes(regs, cand.as_ref())
                .map(|bytes| SearchRes::from_bytes(&bytes, i, cand.as_ref()))
        })
        .collect()
}

fn search_empty<'a, It, I>(candidates: It) -> Vec<SearchRes<'a>>
where
    I: AsRef<str> + 'a,
    It: IntoParallelIterator<Item = &'a I>,
    <It as IntoParallelIterator>::Iter: IndexedParallelIterator,
{
    candidates
        .into_par_iter()
        .enumerate()
        .map(|(i, cand)| SearchRes::empty(i, cand.as_ref()))
        .collect()
}

pub fn search<'a, It, I>(search_term: &str, candidates: It) -> Result<Vec<SearchRes<'a>>>
where
    I: AsRef<str> + 'a,
    It: IntoParallelIterator<Item = &'a I>,
    <It as IntoParallelIterator>::Iter: IndexedParallelIterator,
{
    if search_term.is_empty() {
        Ok(search_empty(candidates))
    } else {
        Ok(search_with_regex(
            &compile_search_term_to_regexes(search_term)?,
            candidates,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_search_simple() {
        let cands = vec!["hej"];
        assert_eq!(
            search("hej", &cands).unwrap(),
            vec![SearchRes::new(0, "hej", vec![0, 1, 2])]
        );

        assert_eq!(search::<_, &str>("hej", &[]).unwrap(), vec![]);

        assert_eq!(search("nej", &cands).unwrap(), vec![]);
    }
}
