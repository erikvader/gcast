use std::{
    env,
    io::{self, BufRead},
};

use lcs::Searcher;

mod lcs;

fn main() {
    let args: Vec<String> = env::args().collect();

    let query = args.get(1).expect("need one argument");

    let input: Result<Vec<String>, _> = io::stdin().lock().lines().into_iter().collect();
    let mut searcher = Searcher::new(input.expect("no stdin"));
    searcher.push_str(query);

    for x in searcher.get_sorted() {
        println!("{}", x.get_compare());
    }
}
