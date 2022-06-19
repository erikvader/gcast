// Taken from els

const CSI: &str = "\x1b[";
const COLOREND: &str = "m";
const SEP: &str = ";";
const RESET: &str = "0";
const BLINK: &str = "5";
const BOLD: &str = "1";
const NORMAL: &str = "22";

macro_rules! color_code {
    (@col black) => {
        "0"
    };
    (@col red) => {
        "1"
    };
    (@col green) => {
        "2"
    };
    (@col yellow) => {
        "3"
    };
    (@col blue) => {
        "4"
    };
    (@col magenta) => {
        "5"
    };
    (@col cyan) => {
        "6"
    };
    (@col white) => {
        "7"
    };
    (bright $($rest:tt)+) => {
        concat!("1;", color_code!($($rest)+))
    };
    (fg $color:tt) => {
        concat!("3", color_code!(@col $color))
    };
    (bg $color:tt) => {
        concat!("4", color_code!(@col $color))
    };
    ($what:tt $color:tt $($rest:tt)+) => {
        concat!(color_code!($what $color), ";", color_code!($($rest)+))
    };
}

#[allow(dead_code)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Color {
    Black = 0,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl Color {
    #![allow(dead_code)]
    pub fn fg_code(self) -> &'static str {
        match self {
            Color::Black => color_code!(fg black),
            Color::Red => color_code!(fg red),
            Color::Green => color_code!(fg green),
            Color::Yellow => color_code!(fg yellow),
            Color::Blue => color_code!(fg blue),
            Color::Magenta => color_code!(fg magenta),
            Color::Cyan => color_code!(fg cyan),
            Color::White => color_code!(fg white),
        }
    }

    pub fn bg_code(self) -> &'static str {
        match self {
            Color::Black => color_code!(bg black),
            Color::Red => color_code!(bg red),
            Color::Green => color_code!(bg green),
            Color::Yellow => color_code!(bg yellow),
            Color::Blue => color_code!(bg blue),
            Color::Magenta => color_code!(bg magenta),
            Color::Cyan => color_code!(bg cyan),
            Color::White => color_code!(bg white),
        }
    }
}

pub trait Len {
    fn char_len(&self) -> usize;
}

impl Len for String {
    fn char_len(&self) -> usize {
        self.as_str().char_len()
    }
}

impl Len for &str {
    fn char_len(&self) -> usize {
        self.chars().count()
    }
}

impl Len for AnsiString {
    fn char_len(&self) -> usize {
        self.length
    }
}

pub struct AnsiString {
    string: String,
    length: usize,
}

impl AnsiString {
    #![allow(dead_code)]
    pub fn new(s: String) -> Self {
        let len = s.char_len();
        AnsiString {
            string: s,
            length: len,
        }
    }

    pub fn empty() -> Self {
        Self::new("".to_string())
    }

    pub fn push_str(&mut self, s: &str) -> &mut Self {
        self.string.push_str(s);
        self.length += s.char_len();
        self
    }

    pub fn push_char(&mut self, s: char) -> &mut Self {
        self.string.push(s);
        self.length += s.len_utf8();
        self
    }

    fn push_ansi(&mut self, s: &str) -> &mut Self {
        self.string.push_str(s);
        self
    }

    fn push_ansi_codes(&mut self, codes: &[&str]) -> &mut Self {
        self.push_ansi(CSI);
        let mut first = true;
        for c in codes {
            if first {
                first = false;
            } else {
                self.push_ansi(SEP);
            }
            self.push_ansi(c);
        }
        self.push_ansi(COLOREND);
        self
    }

    pub fn push_fg(&mut self, col: Color) -> &mut Self {
        self.push_ansi_codes(&[NORMAL, col.fg_code()])
    }

    pub fn push_bg(&mut self, col: Color) -> &mut Self {
        self.push_ansi_codes(&[NORMAL, col.bg_code()])
    }

    pub fn push_fg_bright(&mut self, col: Color) -> &mut Self {
        self.push_ansi_codes(&[BOLD, col.fg_code()])
    }

    pub fn push_bg_bright(&mut self, col: Color) -> &mut Self {
        self.push_ansi_codes(&[BOLD, col.bg_code()])
    }

    pub fn push_reset(&mut self) -> &mut Self {
        self.push_ansi_codes(&[RESET])
    }

    pub fn push_blink(&mut self) -> &mut Self {
        self.push_ansi_codes(&[BLINK])
    }

    pub fn push_code(&mut self, code: &str) -> &mut Self {
        self.push_ansi_codes(&[code])
    }

    pub fn push_maybe_code(&mut self, format_code: Option<&str>) -> &mut Self {
        if let Some(code) = format_code {
            self.push_code(code);
        }
        self
    }

    pub fn push_ansistring(&mut self, other: &AnsiString) -> &mut Self {
        self.length += other.length;
        self.string.push_str(&other.string);
        self
    }
}

impl AsMut<String> for AnsiString {
    fn as_mut(&mut self) -> &mut String {
        &mut self.string
    }
}

impl std::fmt::Display for AnsiString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.string)
    }
}

impl AsRef<str> for AnsiString {
    fn as_ref(&self) -> &str {
        &self.string
    }
}

impl<T> From<T> for AnsiString
where
    T: Into<String>,
{
    fn from(s: T) -> Self {
        AnsiString::new(s.into())
    }
}

pub trait AsString {
    fn as_string(&mut self) -> &mut String;
}

impl AsString for String {
    fn as_string(&mut self) -> &mut String {
        self
    }
}

impl AsString for AnsiString {
    fn as_string(&mut self) -> &mut String {
        &mut self.string
    }
}
