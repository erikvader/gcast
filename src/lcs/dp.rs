use std::num::NonZeroUsize;

trait Recurrence: FnMut(Element, Element, Element) -> Result<Element, ()> {}
impl<T> Recurrence for T where T: FnMut(Element, Element, Element) -> Result<Element, ()> {}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Element {
    length: u8,
    matched: bool,
}

impl Element {
    #[cfg(not(test))]
    pub const MAX: u8 = u8::MAX;
    #[cfg(test)]
    pub const MAX: u8 = 10;

    pub fn empty() -> Self {
        Self {
            length: 0,
            matched: false,
        }
    }

    pub fn matched(self) -> bool {
        self.matched
    }

    pub fn length(self) -> u8 {
        self.length
    }

    pub fn new_matched(other: Self) -> Result<Self, ()> {
        if other.length() < Self::MAX {
            Ok(Self {
                length: other.length() + 1,
                matched: true,
            })
        } else {
            Err(())
        }
    }

    pub fn new_not_matched(other1: Self, other2: Self) -> Self {
        Self {
            length: std::cmp::max(other1.length(), other2.length()),
            matched: false,
        }
    }
}

// Grid ///////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct Grid {
    grid: Vec<Element>,
    height: NonZeroUsize,
}

impl Grid {
    pub fn new_usize(height: usize) -> Self {
        Grid::new(NonZeroUsize::new(height).expect("can't have Grid with height 0"))
    }

    pub fn new(height: NonZeroUsize) -> Self {
        Grid {
            grid: Vec::new(),
            height,
        }
    }

    pub fn len(&self) -> usize {
        self.grid.len()
    }

    pub fn is_empty(&self) -> bool {
        self.grid.is_empty()
    }

    pub fn columns_non_zero(&self) -> usize {
        self.grid.len() / self.height
    }

    pub fn bottom_right(&self) -> (usize, usize) {
        (self.height.get(), self.columns_non_zero())
    }

    pub fn get(&self, row: usize, col: usize) -> Option<Element> {
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

    pub fn get_last(&self) -> Option<Element> {
        self.grid.last().copied()
    }

    pub fn pop_col(&mut self) {
        for _ in 0..self.height.get() {
            self.grid.pop().expect("grid should not be empty");
        }
    }

    pub fn generate_col(&mut self, mut recur: impl Recurrence) -> Result<(), ()> {
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
