use itertools::Itertools;

pub trait Matcher {
    fn push(&mut self, c: char) -> Result<(), ()>;
    fn pop(&mut self);
    fn get_indices(&self) -> Vec<usize>;
    fn get_compare(&self) -> &str;
}

pub fn spread(indices: impl IntoIterator<Item = usize>) -> usize {
    indices
        .into_iter()
        .tuple_windows()
        .map(|(a, b)| b - a - 1)
        .sum()
}

pub fn spread_ref<'a>(indices: impl IntoIterator<Item = &'a usize>) -> usize {
    spread(indices.into_iter().copied())
}

pub fn first_pos(indices: impl IntoIterator<Item = usize>) -> Option<usize> {
    indices.into_iter().next()
}

pub fn first_pos_ref<'a>(indices: impl IntoIterator<Item = &'a usize>) -> Option<usize> {
    indices.into_iter().next().copied()
}
