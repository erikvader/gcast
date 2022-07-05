use colored::*;
use gcast::searcher::Searcher;
use std::{
    env, fs,
    io::{self, BufRead, Write},
    time::{Duration, Instant},
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

    let mut searcher = Searcher::new(file_lines);
    let mut search_time = Duration::from_millis(0);
    loop {
        // reset_term();
        println!("{} {}", "search:".blue(), searcher.get_search());
        println!(
            "{} {}",
            "size:".blue(),
            bytesize::to_string(searcher.size_indication() as u64, false)
        );
        println!("{} {:?}", "time:".blue(), search_time);
        println!();
        let sort_prev = Instant::now();
        for x in searcher.get_sorted_take(10) {
            let indices = x.get_best_indices();
            println!(
                "{}",
                x.get_interspersed(&indices, |c| c.to_string().red(), |c| c)
            );
        }
        println!("{} {:?}", "sort time:".blue(), sort_prev.elapsed());

        let mut line = String::new();
        let bytes_read = io::stdin().read_line(&mut line).expect("read stdin failed");
        if bytes_read == 0 {
            break;
        }

        line = line.trim().into();
        let prev = Instant::now();
        if line.is_empty() {
            searcher.pop();
        } else {
            searcher.push_str(&line).expect("too long");
        }
        search_time = prev.elapsed();
    }
}
