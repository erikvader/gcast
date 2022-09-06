use itertools::Itertools;

#[derive(Debug, PartialEq, Eq)]
pub struct Match {
    indices: Vec<usize>,
}

impl Match {
    pub fn from_vec(mut indices: Vec<usize>) -> Self {
        indices.sort();
        Match { indices }
    }

    pub fn empty() -> Self {
        Self::from_vec(Vec::new())
    }

    pub fn spread(&self) -> usize {
        self.indices
            .iter()
            .tuple_windows()
            .map(|(a, b)| b - a - 1)
            .sum()
    }

    pub fn first(&self) -> Option<usize> {
        self.indices.iter().next().copied()
    }

    pub fn indices(&self) -> &[usize] {
        &self.indices
    }
}
