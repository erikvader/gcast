use super::LCS;
use itertools::Itertools;
use streaming_iterator::StreamingIterator;

// subsequences. There is no guarantee on order, there can be duplicates and it can
// produce exponentially many.
pub struct LCSIterator<'a> {
    lcs: &'a LCS,
    path: Vec<(usize, usize)>,
    dfs: Vec<(usize, usize)>,
}

impl<'a> LCSIterator<'a> {
    pub fn new(lcs: &'a LCS) -> Self {
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
            if cur.length() == left.length() {
                self.dfs.push((row, col - 1));
            }
            if cur.matched() {
                self.dfs.push((row - 1, col - 1));
            }
            if cur.length() == up.length() {
                self.dfs.push((row - 1, col));
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

pub fn path_to_indices(path: &[(usize, usize)]) -> impl Iterator<Item = usize> + '_ {
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
