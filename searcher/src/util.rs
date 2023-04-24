use std::ops::Range;

fn compact_to_ranges(indices: &[usize]) -> Vec<Range<usize>> {
    let mut iter = indices.iter();
    if let Some(&first) = iter.next() {
        let mut res = vec![Range {
            start: first,
            end: first + 1,
        }];

        iter.for_each(|&x| {
            let end = &mut res.last_mut().unwrap().end;
            assert!(x >= *end, "must be strictly increasing");
            if *end == x {
                *end = x + 1;
            } else {
                res.push(Range {
                    start: x,
                    end: x + 1,
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
    assert_eq!(compact_to_ranges(&[]), Vec::<Range<usize>>::new());
    assert_eq!(compact_to_ranges(&[1, 2, 3]), vec![1..4]);
    assert_eq!(compact_to_ranges(&[1, 3, 4]), vec![1..2, 3..5]);
    assert_eq!(
        compact_to_ranges(&[1, 3, 7, 9]),
        vec![1..2, 3..4, 7..8, 9..10]
    );
}

#[test]
#[should_panic(expected = "must be strictly increasing")]
fn test_compact_invalid_input() {
    compact_to_ranges(&[1, 1]);
}

// Preconditions:
//   - ranges only contains non-empty ranges
//   - ranges is non-overlapping
//   - ranges is increasing
//   - cover includes all ranges
fn complement<T>(ranges: &[Range<T>], cover: Range<T>) -> Vec<Range<T>>
where
    T: Clone,
{
    let mut res = Vec::new();
    let mut start = cover.start;
    for r in ranges {
        res.push(start.clone()..r.start.clone());
        start = r.end.clone();
    }
    res.push(start..cover.end);
    res
}

#[test]
fn test_range_complement() {
    assert_eq!(complement(&[], 1..6), vec![1..6]);
    assert_eq!(complement(&[0..2], 0..5), vec![0..0, 2..5]);
    assert_eq!(complement(&[1..2], 0..5), vec![0..1, 2..5]);
    assert_eq!(complement(&[1..2, 2..3], 0..5), vec![0..1, 2..2, 3..5]);
}

pub fn stylize<T, ON, OFF, Res>(
    string: &str,
    indices: &[usize],
    on_lcs: ON,
    off_lcs: OFF,
) -> Res
where
    ON: Fn(&str) -> T,
    OFF: Fn(&str) -> T,
    Res: FromIterator<T>,
{
    let colored = compact_to_ranges(indices);
    let uncolored = complement(&colored, 0..string.chars().count());
    assert!(colored.len() + 1 == uncolored.len());

    let mut res = Vec::new();
    let mut uc_iter = uncolored.into_iter();
    for c in colored {
        let uc = uc_iter.next().expect("will work if lengths are correct");
        if !uc.is_empty() {
            res.push(off_lcs(slice_chars(string, uc)));
        }

        if !c.is_empty() {
            res.push(on_lcs(slice_chars(string, c)));
        }
    }

    let uc = uc_iter.next().expect("should be one more");
    if !uc.is_empty() {
        res.push(off_lcs(slice_chars(string, uc)));
    }

    res.into_iter().collect()
}

#[test]
fn test_stylize() {
    assert_eq!(
        stylize::<_, _, _, String>("hej", &[1], |_| 'a', |_| 'b'),
        "bab"
    );

    assert_eq!(stylize::<_, _, _, String>("", &[], |_| 'a', |_| 'b'), "");

    assert_eq!(
        stylize::<_, _, _, String>("hej", &[0, 1, 2], |_| 'a', |_| 'b'),
        "a"
    );
    assert_eq!(
        stylize::<_, _, _, String>("hej", &[], |_| 'a', |_| 'b'),
        "b"
    );

    let path = "/ständiga frågan  www.instagram.com_uumemes-461225481079299.mp4";
    assert_eq!(
        stylize::<_, _, _, String>(path, &[], |x| x.to_string(), |x| x.to_string()),
        path
    );
}

// TODO: Use something better. Is there a type from a crate that can operate efficiently
// on chars?
fn slice_chars(string: &str, slice: Range<usize>) -> &str {
    let mut bytes: Vec<_> = string.char_indices().map(|(i, _)| i).collect();
    bytes.push(string.len());
    let s = bytes.get(slice.start..slice.end + 1).unwrap_or_else(|| {
        panic!(
            "slice out of bounds in slice_chars, string: '{}', slice: {:?}",
            string, slice
        )
    });
    &string[*s.first().unwrap()..*s.last().unwrap()]
}

#[test]
fn test_slice_chars() {
    let s = "åäöÅÄÖ";
    assert_eq!(s.len(), 2 * s.chars().count());
    assert_eq!(slice_chars(s, 0..1), "å");
    assert_eq!(slice_chars(s, 0..2), "åä");
    assert_eq!(slice_chars(s, 1..2), "ä");
    assert_eq!(slice_chars(s, 0..6), s);
    assert_eq!(slice_chars(s, 0..5), "åäöÅÄ");

    assert_eq!(slice_chars("", 0..0), "");
    assert_eq!(slice_chars(s, 0..0), "");
    assert_eq!(slice_chars(s, 4..4), "");
}

#[test]
#[should_panic]
fn test_slice_chars_out_of_range() {
    assert_eq!(slice_chars("", 0..1), "");
}

pub fn sorted_take<T>(items: &mut [T], len: usize) -> &mut [T]
where
    T: Ord,
{
    if len == 0 {
        return &mut [][..];
    }
    if len < items.len() {
        let (beg, _, _) = items.select_nth_unstable(len);
        beg.sort_unstable();
        beg
    } else {
        items.sort_unstable();
        items
    }
}

#[test]
fn test_sorted_take() {
    let mut list = vec![6, 4, 3, 1];
    assert_eq!(sorted_take(&mut list, 2), &[1, 3]);

    assert_eq!(sorted_take(&mut list, 0), &[]);

    assert_eq!(sorted_take(&mut list, 10), &[1, 3, 4, 6]);

    assert_eq!(sorted_take::<()>(&mut [], 10), &[]);
}
