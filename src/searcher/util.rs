use std::ops::Range;

pub fn compact_to_ranges<T, It>(verbose: It, one: T) -> Vec<Range<T>>
where
    T: PartialOrd<T> + Copy + std::ops::Add<Output = T>,
    It: IntoIterator<Item = T>,
{
    assert!(one + one > one, "unexpected behaviour of 'one'");
    let mut iter = verbose.into_iter();
    if let Some(first) = iter.next() {
        let mut res = vec![Range {
            start: first,
            end: first + one,
        }];

        iter.for_each(|x| {
            let end = &mut res.last_mut().unwrap().end;
            assert!(x >= *end, "must be strictly increasing");
            if *end == x {
                *end = x + one;
            } else {
                res.push(Range {
                    start: x,
                    end: x + one,
                });
            }
        });
        res
    } else {
        Vec::new()
    }
}

#[test]
fn test_compact_ranges() {
    assert_eq!(compact_to_ranges(vec![], 1), vec![]);
    assert_eq!(compact_to_ranges(vec![1, 2, 3], 1), vec![1..4]);
    assert_eq!(compact_to_ranges(vec![1, 3, 4], 1), vec![1..2, 3..5]);
    assert_eq!(
        compact_to_ranges(vec![1, 3, 7, 9], 1),
        vec![1..2, 3..4, 7..8, 9..10]
    );
}

#[test]
#[should_panic(expected = "must be strictly increasing")]
fn test_compact_invalid_input() {
    compact_to_ranges(vec![1, 1], 1);
}

pub fn get_interspersed<T1, T2, ON, OFF>(
    string: &str,
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
    for (i, c) in string.chars().enumerate() {
        if indices.binary_search(&i).is_ok() {
            res.push_str(&on_lcs(c).to_string());
        } else {
            res.push_str(&off_lcs(c).to_string());
        }
    }
    res
}
