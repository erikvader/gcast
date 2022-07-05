use itertools::Itertools;
use streaming_iterator::StreamingIterator;

use self::{
    dp::{Element, Grid},
    iterator::{path_to_indices, LCSIterator},
};

pub mod dp;
pub mod iterator;
pub mod util;

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
            if c.eq_ignore_ascii_case(&cur) {
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

    pub fn get_leftmost_indices(&self) -> Vec<usize> {
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

    pub fn spread(&self, indices: impl IntoIterator<Item = usize>) -> usize {
        indices
            .into_iter()
            .tuple_windows()
            .map(|(a, b)| b - a - 1)
            .sum()
    }

    pub fn spread_ref<'a>(&self, indices: impl IntoIterator<Item = &'a usize>) -> usize {
        self.spread(indices.into_iter().copied())
    }

    pub fn first_pos<'a>(
        &self,
        indices: impl IntoIterator<Item = &'a usize>,
    ) -> Option<usize> {
        indices.into_iter().next().copied()
    }

    pub fn grid_len(&self) -> usize {
        self.dp.len()
    }

    pub fn grid_columns(&self) -> usize {
        self.dp.columns_non_zero()
    }
}

#[test]
fn test_lcs() {
    let mut lcs = LCS::new("GAC".into());
    assert!(lcs.push_str("AGCAT").is_ok());
    assert_eq!(lcs.length(), 2);
    let indices = lcs.get_leftmost_indices();
    assert!(vec!["AC", "GC", "GA"].contains(&lcs.get_string(&indices).as_str()));
}

#[test]
fn test_empty_lcs() {
    let lcs = LCS::new("asd".into());
    let indices = lcs.get_leftmost_indices();
    assert!(indices.is_empty());
    assert!(lcs.get_string(&indices).is_empty());
}

#[test]
fn test_lcs_intersperse() {
    let mut lcs = LCS::new("asd".into());
    assert!(lcs.push('s').is_ok());
    let indices = lcs.get_leftmost_indices();
    assert_eq!(
        lcs.get_interspersed(&indices, |c| format!("1{}2", c), |c| c),
        "a1s2d"
    );
}

#[test]
fn test_lcs_all_subsequences() {
    let mut lcs = LCS::new("src/lc".into());
    assert!(lcs.push_str("src").is_ok());
    assert_eq!(lcs.length(), 3);

    let all: Vec<Vec<usize>> = lcs.get_all_indices().sorted().dedup().collect();
    assert_eq!(all, vec![vec![0, 1, 2], vec![0, 1, 5]]);
    assert!(all.contains(&lcs.get_leftmost_indices()));

    let strings: Vec<String> =
        all.iter().map(|indices| lcs.get_string(indices)).collect();
    assert_eq!(strings, vec!["src", "src"]);

    let first_poses: Vec<Option<usize>> =
        all.iter().map(|indices| lcs.first_pos(indices)).collect();
    assert_eq!(first_poses, vec![Some(0), Some(0)]);

    let spreads: Vec<usize> = all.iter().map(|indices| lcs.spread_ref(indices)).collect();
    assert_eq!(spreads, vec![0, 3]);
}

#[test]
fn test_lcs_leftmost() {
    let mut lcs = LCS::new("scsrrcc/src".into());
    assert!(lcs.push_str("src").is_ok());
    assert_eq!(lcs.length(), 3);

    let left = lcs.get_leftmost_indices();
    assert_eq!(left, vec![0, 3, 5]);
    assert_eq!(lcs.spread(left), 3);
}

#[test]
fn test_lcs_pop() {
    let mut lcs = LCS::new("asd".into());
    assert!(lcs.push('s').is_ok());
    assert!(lcs.push('d').is_ok());
    let indices = lcs.get_leftmost_indices();
    assert_eq!(lcs.get_string(&indices), "sd");

    lcs.pop();
    let indices = lcs.get_leftmost_indices();
    assert_eq!(lcs.get_string(&indices), "s");
}
