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

    pub fn read_input(&mut self) -> (Input, State) {
        input! {
            from &mut self.source,
            n: usize,
            m: usize,
            k: usize,
            t: usize,
            cards: [(usize, i64); n],
            projects: [(i64, i64); m],
        }
        let cards = cards
            .iter()
            .copied()
            .map(|(t, w)| (CardType::from_usize(t), w))
            .collect();
        (
            Input { n, m, k, t },
            State {
                score: 0,
                cards,
                projects,
            },
        )
    }

    pub fn output_c(&self, c: usize, m: usize) {
        println!("{} {}", c, m);
        self.flush();
    }

    pub fn output_r(&self, r: usize) {
        println!("{}", r);
        self.flush();
    }

    pub fn read_status(&mut self, input: &Input) -> (Vec<(i64, i64)>, i64, Vec<(usize, i64, i64)>) {
        input! {
            from &mut self.source,
            hv: [(i64, i64); input.m],
            money: i64,
            twp: [(usize, i64, i64); input.k],
        }
        (hv, money, twp)
    }

    fn flush(&self) {
        std::io::stdout().flush().unwrap();
    }
}
