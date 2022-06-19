mod ansistring;
mod lcs;

use std::{
    env,
    io::{self, BufRead},
};

use ansistring::{AnsiString, Color};
use lcs::Searcher;

fn main() {
    let args: Vec<String> = env::args().collect();

    let query = args.get(1).expect("need one argument");

    let input: Result<Vec<String>, _> = io::stdin().lock().lines().into_iter().collect();
    let mut searcher = Searcher::new(input.expect("no stdin"));
    searcher.push_str(query);

    let left = AnsiString::empty().push_fg_bright(Color::Red).to_string();
    let right = AnsiString::empty().push_reset().to_string();
    for x in searcher.get_sorted() {
        println!("{}", x.get_interspersed(&left, &right));
    }
}
