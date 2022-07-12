use std::collections::HashSet;

use regex::Regex;

use self::{
    compile::{compile_search_term_to_regex, Result},
    r#match::Match,
};

mod compile;
mod r#match;
pub mod util;

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
    It: IntoIterator<Item = &'a I>,
{
    let mut res = Vec::new();
    for (i, cand) in candidates.into_iter().enumerate() {
        if let Some(bytes) = run_regexes_get_bytes(regs, cand.as_ref()) {
            res.push(SearchRes::from_bytes(&bytes, i, cand.as_ref()));
        }
    }
    res
}

fn search_empty<'a, It, I>(candidates: It) -> Vec<SearchRes<'a>>
where
    I: AsRef<str> + 'a,
    It: IntoIterator<Item = &'a I>,
{
    candidates
        .into_iter()
        .enumerate()
        .map(|(i, cand)| SearchRes::empty(i, cand.as_ref()))
        .collect()
}

pub fn search<'a, It, I>(search_term: &str, candidates: It) -> Result<Vec<SearchRes<'a>>>
where
    I: AsRef<str> + 'a,
    It: IntoIterator<Item = &'a I>,
{
    if search_term.is_empty() {
        Ok(search_empty(candidates))
    } else {
        Ok(search_with_regex(
            &compile_search_term_to_regex(search_term)?,
            candidates,
        ))
    }
}

#[test]
fn test_search() {
    let cands: Vec<String> = vec!["hej".to_string()];
    search("hej", &cands).unwrap();
}

// TODO: create tests from these
//#[cfg(test)]
//mod test {
//     use super::*;

//     impl Searcher {
//         fn assert_invariants(&self) {
//             assert_eq!(self.num_chars, self.search.chars().count());
//             assert!(self
//                 .active
//                 .iter()
//                 .all(|x| x.lcs.grid_columns() == self.num_chars));
//             assert!(self.active.iter().all(|x| x.lcs.length() == self.num_chars));
//             assert!(!self
//                 .inactive
//                 .iter()
//                 .any(|x| x.lcs.grid_columns() > self.num_chars));
//             assert!(!self
//                 .inactive
//                 .iter()
//                 .any(|x| x.lcs.length() >= self.num_chars));
//         }
//     }

//     #[test]
//     fn test_lcs_searcher() {
//         let mut searcher = Searcher::new(vec!["aaa", "aab", "aa", "abab", "bbbb"]);
//         assert_eq!(searcher.get_sorted().count(), 5);
//         assert_eq!(searcher.size_indication(), 0, "all grids should be empty");
//         assert_eq!(searcher.get_search(), "");
//         searcher.assert_invariants();

//         assert!(searcher.push('a').is_ok());
//         searcher.assert_invariants();
//         assert_eq!(searcher.get_sorted().count(), 4);
//         assert_eq!(searcher.get_search(), "a");

//         searcher.pop();
//         searcher.assert_invariants();
//         assert_eq!(searcher.get_sorted().count(), 5);
//         assert_eq!(searcher.get_search(), "");
//         assert_eq!(searcher.size_indication(), 0, "all grids should be empty");

//         assert!(searcher.push_str("aab").is_ok());
//         searcher.assert_invariants();
//         assert_eq!(searcher.get_sorted().count(), 2);

//         searcher.pop();
//         searcher.assert_invariants();
//         assert_eq!(searcher.get_sorted().count(), 4);

//         searcher.pop();
//         searcher.assert_invariants();
//         assert_eq!(searcher.get_sorted().count(), 4);

//         searcher.pop();
//         searcher.assert_invariants();
//         assert_eq!(searcher.get_sorted().count(), 5);
//         assert_eq!(searcher.get_search(), "");
//         assert_eq!(searcher.size_indication(), 0, "all grids should be empty");
//     }

//     #[test]
//     fn test_lcs_searcher_too_long() {
//         let s = "aaaaaaaaaaa";
//         assert!(s.chars().count() > Element::MAX.into());
//         let mut searcher = Searcher::new(vec![s]);
//         assert_eq!(Err(10), searcher.push_str(s));
//         assert_eq!(&s[..10], searcher.get_search());
//         assert_eq!(10, searcher.num_chars);
//     }

//     #[test]
//     fn test_lcs_searcher_empty() {
//         let mut searcher = Searcher::new::<String, Vec<String>>(Vec::new());
//         assert!(searcher.push('a').is_ok());
//         searcher.assert_invariants();
//         assert_eq!(searcher.get_sorted().count(), 0);
//         assert_eq!(searcher.get_search(), "a");
//     }

//     #[test]
//     fn test_lcs_searcher_longer() {
//         let mut searcher = Searcher::new(vec!["ab"]);
//         assert_eq!(searcher.push_str("abb"), Ok(()));
//         searcher.assert_invariants();
//         assert_eq!(searcher.get_search(), "abb");
//         assert_eq!(searcher.get_sorted().count(), 0);
//         assert_eq!(searcher.get_sorted_take(10).count(), 0);

//         searcher.pop();
//         searcher.assert_invariants();
//         assert_eq!(searcher.get_search(), "ab");
//         assert_eq!(searcher.get_sorted().count(), 1);
//     }
// }
