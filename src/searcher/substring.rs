use std::rc::Rc;

pub struct SubStr<T> {
    compare: T,
}

impl<T> SubStr<T>
where
    T: AsRef<str>,
{
    pub fn new(compare: T) -> Self {
        Self { compare }
    }
}

#[test]
fn test_substr() {
    let string = Rc::from("asd".to_string());
    let cmp = SubStr::new(string);
    let cmp2 = SubStr::new("asd".to_string());
}
