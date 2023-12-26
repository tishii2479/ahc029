use crate::def::*;
use std::io::{Stdin, Write};

use proconio::*;

pub trait Interactor {
    fn output_c(&mut self, c: usize, m: usize);
    fn output_r(&mut self, r: usize);
    fn read_status(&mut self, input: &Input) -> (Vec<Project>, i64, Vec<(Card, i64)>);
}

pub struct IOInteractor {
    source: proconio::source::line::LineSource<std::io::BufReader<Stdin>>,
}

impl IOInteractor {
    pub fn new() -> IOInteractor {
        IOInteractor {
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
            hv: [(i64, i64); m],
        }
        let projects = hv.iter().copied().map(|(h, v)| Project { h, v }).collect();
        let cards = cards
            .iter()
            .copied()
            .map(|(t, w)| Card::from_tw(t, w))
            .collect();
        (
            Input { n, m, k, t },
            State {
                last_invest_round: 0,
                invest_level: 0,
                score: 0,
                cards,
                projects,
            },
        )
    }

    fn flush(&self) {
        std::io::stdout().flush().unwrap();
    }
}

impl Interactor for IOInteractor {
    fn output_c(&mut self, c: usize, m: usize) {
        println!("{} {}", c, m);
        self.flush();
    }

    fn output_r(&mut self, r: usize) {
        println!("{}", r);
        self.flush();
    }

    fn read_status(&mut self, input: &Input) -> (Vec<Project>, i64, Vec<(Card, i64)>) {
        input! {
            from &mut self.source,
            hv: [(i64, i64); input.m],
            money: i64,
            twp: [(usize, i64, i64); input.k],
        }
        let projects = hv.iter().copied().map(|(h, v)| Project { h, v }).collect();
        let cards: Vec<(Card, i64)> = twp
            .iter()
            .copied()
            .map(|(t, w, p)| (Card::from_tw(t, w), p))
            .collect();
        (projects, money, cards)
    }
}
