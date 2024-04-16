use std::collections::HashSet;

use regex::Regex;

use self::{
    compile::{compile_search_term_to_regexes, Result},
    r#match::Match,
};

mod compile;
mod r#match;

pub use compile::CompileError;

#[derive(Debug)]
pub struct SearchRes<T> {
    mat: Match,
    index: usize,
    inner: T,
}

impl<T> Ord for SearchRes<T> {
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

impl<T> PartialOrd for SearchRes<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for SearchRes<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl<T> Eq for SearchRes<T> {}

impl<T> SearchRes<T> {
    pub fn get_inner(&self) -> &T {
        &self.inner
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_match(&self) -> &Match {
        &self.mat
    }
}

impl<T> SearchRes<T> {
    fn from_bytes(bytes: &HashSet<usize>, index: usize, string: T) -> Self
    where
        T: AsRef<str>,
    {
        let mat = Match::from_vec(
            string
                .as_ref()
                .char_indices()
                .map(|(b, _)| b)
                .enumerate()
                .filter(|(_, b)| bytes.contains(b))
                .map(|(i, _)| i)
                .collect(),
        );
        SearchRes {
            index,
            inner: string,
            mat,
        }
    }

    fn empty(index: usize, string: T) -> Self {
        SearchRes {
            index,
            inner: string,
            mat: Match::empty(),
        }
    }
}

fn run_regexes_get_bytes(regs: &[Regex], string: &str) -> Option<HashSet<usize>> {
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

fn search_with_regex<'a, It, T>(regs: &[Regex], candidates: It) -> Vec<SearchRes<T>>
where
    T: AsRef<str>,
    It: IntoIterator<Item = T>,
{
    candidates
        .into_iter()
        .enumerate()
        .filter_map(|(i, cand)| {
            run_regexes_get_bytes(regs, cand.as_ref())
                .map(|bytes| SearchRes::from_bytes(&bytes, i, cand))
        })
        .collect()
}

fn search_empty<'a, It, T>(candidates: It) -> Vec<SearchRes<T>>
where
    It: IntoIterator<Item = T>,
{
    candidates
        .into_iter()
        .enumerate()
        .map(|(i, cand)| SearchRes::empty(i, cand))
        .collect()
}

pub fn search<'a, It, T>(search_term: &str, candidates: It) -> Result<Vec<SearchRes<T>>>
where
    T: AsRef<str>,
    It: IntoIterator<Item = T>,
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

    impl SearchRes<&'static str> {
        fn new(index: usize, string: &'static str, indices: Vec<usize>) -> Self {
            SearchRes {
                index,
                inner: string,
                mat: Match::from_vec(indices),
            }
        }
    }

    #[test]
    fn test_search_simple() {
        let cands = vec!["hej"];
        assert_eq!(
            search("hej", cands.clone()).unwrap(),
            vec![SearchRes::new(0, "hej", vec![0, 1, 2])]
        );

        assert_eq!(search::<&[&'static str], _>("hej", &[]).unwrap(), vec![]);

        assert_eq!(search("nej", cands).unwrap(), vec![]);
    }
}
