#![allow(dead_code)] // NOTE: this is just quick testing

use colored::*;
use searcher::{search, sorted_take, stylize};
use std::{
    env, fs,
    io::{self, Write},
    time::Instant,
};

const CSI: &str = "\x1b[";

fn reset_term() {
    print!("{}1;1H", CSI);
    print!("{}2J", CSI);
    io::stdout().flush().unwrap();
}

struct TermSaver;
impl TermSaver {
    fn new() -> Self {
        print!("{}?1049h", CSI);
        io::stdout().flush().unwrap();
        TermSaver
    }
}

impl Drop for TermSaver {
    fn drop(&mut self) {
        print!("{}?1049l", CSI);
        io::stdout().flush().unwrap();
    }
}

fn main() {
    let file_lines: Vec<String> = {
        let args: Vec<String> = env::args().collect();
        let file = args.get(1).expect("need one argument");
        let file_contents = fs::read_to_string(file).expect("could not read file");
        file_contents
            .lines()
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string())
            .collect()
    };

    // let _cookie = TermSaver::new();

    let mut search_term = String::new();
    loop {
        // reset_term();
        println!("{} {}", "search:".blue(), search_term);
        // println!(
        //     "{} {:?}",
        //     "regex:".blue(),
        //     gcast::searcher::compile::compile_search_term_to_strings(&search_term)
        // );
        let search_prev = Instant::now();
        if let Ok(mut search_res) = search(&search_term, &file_lines) {
            println!("{} {:?}", "search time:".blue(), search_prev.elapsed());

            let sort_prev = Instant::now();
            let first_ten = sorted_take(&mut search_res, 10);
            println!("{} {:?}", "sort time:".blue(), sort_prev.elapsed());

            for x in first_ten.iter_mut() {
                println!(
                    "{}",
                    stylize::<_, _, _, String>(
                        x.get_inner(),
                        x.get_match().indices(),
                        |c| c.to_string().red().to_string(),
                        |c| c.to_string()
                    )
                );
            }
        } else {
            println!("{}", "invalid search term".red());
        }

        let mut line = String::new();
        let bytes_read = io::stdin().read_line(&mut line).expect("read stdin failed");
        if bytes_read == 0 {
            break;
        }

        line = line.trim_end().into();
        if line.is_empty() {
            search_term.pop();
        } else {
            search_term += &line;
        }
    }
}
