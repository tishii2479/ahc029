use crate::def::*;
use std::io::{Stdin, Write};

use proconio::*;

pub struct Interactor {
    source: proconio::source::line::LineSource<std::io::BufReader<Stdin>>,
}

impl Interactor {
    pub fn new() -> Interactor {
        Interactor {
            source: proconio::source::line::LineSource::new(std::io::BufReader::new(
                std::io::stdin(),
            )),
        }
    }

    pub fn read_input(&mut self) -> Input {
        input! {
            from &mut self.source,
            d: usize,
            n: usize,
            blocks: [(usize,usize); n],
        }
        Input { d, n, blocks }
    }

    pub fn output_p(&self, p: (usize, usize)) {
        println!("{} {}", p.0, p.1);
        self.flush();
    }

    pub fn read_t(&mut self) -> usize {
        input! {
            from &mut self.source,
            t: usize
        }
        t
    }

    pub fn output_q(&self, q: &Vec<(i32, (usize, usize))>) {
        for (_, (y, x)) in q {
            println!("{} {}", y, x);
        }
        self.flush();
    }

    fn flush(&self) {
        std::io::stdout().flush().unwrap();
    }
}
