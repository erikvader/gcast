use regex::Regex;
use regex_syntax::{escape, escape_into};

const QUOTE_WORD: &str = "'";

const REG_GROUP_START: &str = "(";
const REG_GROUP_END: &str = ")";
const REG_ANY: &str = ".*?";
const REG_ICASE: &str = "(?i)";
const REG_NO_ICASE: &str = "(?-i)";

// TODO: create an error struct and report what went wrong in more detail
pub type Result<T> = std::result::Result<T, ()>;

pub fn compile_search_term_to_regexes(search_term: &str) -> Result<Vec<Regex>> {
    compile_search_term_to_strings(search_term).map(|v| {
        v.into_iter()
            .map(|reg_str| {
                Regex::new(&reg_str).expect("the regex should always be correct")
            })
            .collect()
    })
}

fn compile_search_term_to_strings(search_term: &str) -> Result<Vec<String>> {
    let regs: Result<Vec<_>> = search_term
        .split_whitespace()
        .map(|word| compile_word(word))
        .collect();

    regs.and_then(|vec| if vec.is_empty() { Err(()) } else { Ok(vec) })
}

fn compile_word(word: &str) -> Result<String> {
    assert!(!word.is_empty());
    let reg_str = if let Some(w) = word.strip_prefix(QUOTE_WORD) {
        if w.is_empty() {
            return Err(());
        } else {
            literal_word(w)
        }
    } else {
        fuzzy_word(word)
    };
    Ok(reg_str)
}

fn smart_case(word: &str) -> &'static str {
    assert!(!word.is_empty());
    if word.chars().any(|c| c.is_uppercase()) {
        REG_NO_ICASE
    } else {
        REG_ICASE
    }
}

fn literal_word(word: &str) -> String {
    assert!(!word.is_empty());
    String::new()
        + smart_case(word)
        + REG_ANY
        + REG_GROUP_START
        + &escape(word)
        + REG_GROUP_END
        + REG_ANY
}

fn fuzzy_word(word: &str) -> String {
    assert!(!word.is_empty());
    let mut fuzz = String::new() + smart_case(word) + REG_ANY;
    for (b, c) in word.char_indices() {
        let s = &word[b..b + c.len_utf8()];
        fuzz += REG_GROUP_START;
        escape_into(s, &mut fuzz);
        fuzz += REG_GROUP_END;
        fuzz += REG_ANY;
    }
    fuzz
}

#[test]
fn test_compile() {
    assert_eq!(literal_word("a?"), "(?i).*?(a\\?).*?");
    assert_eq!(fuzzy_word("a?"), "(?i).*?(a).*?(\\?).*?");
    assert!(compile_word("'a").is_ok());
    assert!(compile_word("'").is_err());

    let regs = compile_search_term_to_regexes("'a asd");
    assert!(regs.is_ok());
    assert_eq!(regs.unwrap().len(), 2);
}
