use regex::Regex;
use regex_syntax::{escape, escape_into};

const QUOTE_WORD: &str = "'";

const REG_GROUP_START: &str = "(";
const REG_GROUP_END: &str = ")";
const REG_ANY: &str = ".*?";
const REG_ICASE: &str = "(?i)";
const REG_NO_ICASE: &str = "(?-i)";

// TODO: report what went wrong in more detail
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("failed to compile query to regex")]
pub struct CompileError;

pub type Result<T> = std::result::Result<T, CompileError>;

pub fn compile_search_term_to_regexes(search_term: &str) -> Result<Vec<Regex>> {
    compile_swiper(search_term).map(|v| {
        v.into_iter()
            .map(|reg_str| {
                Regex::new(&reg_str).expect("the regex should always be correct")
            })
            .collect()
    })
}

#[allow(dead_code)] // NOTE: maybe reintroduce in the future as a setting or something
fn compile_fzf(search_term: &str) -> Result<Vec<String>> {
    let regs: Result<Vec<_>> = search_term.split_whitespace().map(compile_word).collect();

    regs.and_then(|vec| {
        if vec.is_empty() {
            Err(CompileError)
        } else {
            Ok(vec)
        }
    })
}

fn compile_swiper(mut search_term: &str) -> Result<Vec<String>> {
    if Regex::new(r"(^ ?$)|(^ [^ ])")
        .unwrap()
        .is_match(search_term)
    {
        return Err(CompileError);
    }

    if Regex::new(r"[^ ] $").unwrap().is_match(search_term) {
        search_term = search_term.trim_end_matches(' ');
    }

    let parts: Vec<_> = Regex::new(r"( +)|[^ ]+")
        .unwrap()
        .find_iter(search_term)
        .map(|m| {
            if m.as_str().starts_with(' ') {
                swiper_space(m.as_str())
            } else {
                swiper_word(m.as_str())
            }
        })
        .collect();

    assert!(
        !parts.is_empty(),
        "if search_term is non-empty, then this must contain something"
    );

    let regstring: String = std::iter::once(smart_case(search_term).to_string())
        .into_iter()
        .chain(parts)
        .collect();
    Ok(vec![regstring])
}

fn compile_word(word: &str) -> Result<String> {
    assert!(!word.is_empty());
    let reg_str = if let Some(w) = word.strip_prefix(QUOTE_WORD) {
        if w.is_empty() {
            return Err(CompileError);
        } else {
            literal_word(w)
        }
    } else {
        fuzzy_word(word)
    };
    Ok(reg_str)
}

fn swiper_space(spaces: &str) -> String {
    match spaces.chars().count() {
        x if x == 0 => panic!("must be non-empty"),
        1 => REG_ANY.into(),
        x => format!(r" {{{}}}", x - 1),
    }
}

fn swiper_word(word: &str) -> String {
    assert!(!word.is_empty());
    String::new() + REG_GROUP_START + &escape(word) + REG_GROUP_END
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
fn test_compile_fzf() {
    assert_eq!(literal_word("a?"), "(?i).*?(a\\?).*?");
    assert_eq!(fuzzy_word("a?"), "(?i).*?(a).*?(\\?).*?");
    assert!(compile_word("'a").is_ok());
    assert!(compile_word("'").is_err());

    let regs = compile_fzf("'a asd");
    assert!(regs.is_ok());
    assert_eq!(regs.unwrap().len(), 2);
}

#[test]
fn test_compile_swiper() {
    assert_eq!(compile_swiper(" "), Err(CompileError));
    assert_eq!(compile_swiper(""), Err(CompileError));
    assert_eq!(compile_swiper(" x"), Err(CompileError));
    assert!(compile_swiper("x ").is_ok());
    assert!(compile_swiper("x  ").is_ok());

    assert_eq!(
        compile_swiper("hej hej"),
        Ok(vec![format!("{}(hej){}(hej)", REG_ICASE, REG_ANY)])
    );
    assert_eq!(
        compile_swiper("hej  hej"),
        Ok(vec![format!("{}(hej) {{1}}(hej)", REG_ICASE)])
    );
    assert_eq!(
        compile_swiper("hej    hej"),
        Ok(vec![format!("{}(hej) {{3}}(hej)", REG_ICASE)])
    );
    assert_eq!(
        compile_swiper("hEj"),
        Ok(vec![format!("{}(hEj)", REG_NO_ICASE)])
    );
}
