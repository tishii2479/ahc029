pub const MAX_INVEST_LEVEL: usize = 20;
pub const INF: f64 = 1e18;

use crate::interactor::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Card {
    WorkSingle(i64),
    WorkAll(i64),
    CancelSingle,
    CancelAll,
    Invest,
    None,
}

#[derive(Debug, Clone, Copy)]
pub struct Project {
    pub h: i64,
    pub v: i64,
}

impl Card {
    pub fn from_tw(t: usize, w: i64) -> Card {
        match t {
            0 => Card::WorkSingle(w),
            1 => Card::WorkAll(w),
            2 => Card::CancelSingle,
            3 => Card::CancelAll,
            4 => Card::Invest,
            5 => Card::None,
            _ => panic!("invalid card type: {t}"),
        }
    }

    pub fn to_t(&self) -> usize {
        match self {
            Card::WorkSingle(_) => 0,
            Card::WorkAll(_) => 1,
            Card::CancelSingle => 2,
            Card::CancelAll => 3,
            Card::Invest => 4,
            Card::None => panic!(),
        }
    }
}

pub struct Input {
    pub n: usize,
    pub m: usize,
    pub k: usize,
    pub t: usize,
}

#[derive(Clone, Copy)]
pub struct Param {
    pub overflow_alpha_refill: f64,
    pub overflow_alpha: f64,
    pub overflow_alpha_all_refill: f64,
    pub overflow_alpha_all: f64,
    pub cancel_alpha_refill: f64,
    pub cancel_alpha: f64,
    pub cancel_alpha_all_refill: f64,
    pub cancel_alpha_all: f64,
    pub invest_limit: usize,
    pub cancel_limit: usize,
    pub invest_cost: i64,
    pub work_single_beta: f64,
}

#[derive(Debug, Clone)]
pub struct State {
    pub last_invest_round: usize,
    pub invest_level: usize,
    pub score: i64,
    pub cards: Vec<Card>,
    pub projects: Vec<Project>,
}

impl State {
    pub fn read_status<I: Interactor>(
        &mut self,
        input: &Input,
        interactor: &mut I,
    ) -> Vec<(Card, i64)> {
        let (projects, score, new_cards) = interactor.read_status(input);
        self.projects = projects;
        self.score = score;
        new_cards
    }

    pub fn use_card<I: Interactor>(&mut self, use_card: usize, m: usize, interactor: &mut I) {
        interactor.output_c(use_card, m);
        if let Card::Invest = self.cards[use_card] {
            self.invest_level += 1;
        }
        self.cards[use_card] = Card::None;
    }

    pub fn refill_card<I: Interactor>(
        &mut self,
        selected_card: usize,
        new_cards: &Vec<(Card, i64)>,
        interactor: &mut I,
    ) {
        interactor.output_r(selected_card);
        let i = self.empty_card_index().unwrap();
        self.cards[i] = new_cards[selected_card].0;
    }

    pub fn empty_card_index(&self) -> Option<usize> {
        for i in 0..self.cards.len() {
            if let Card::None = self.cards[i] {
                return Some(i);
            }
        }
        None
    }
}

pub struct Recorder {
    pub scores: Vec<i64>,
    pub invest_rounds: Vec<usize>,
    pub x: Vec<i64>,
}

impl Recorder {
    pub fn new() -> Recorder {
        Recorder {
            scores: vec![0],
            invest_rounds: vec![],
            x: vec![0; 5],
        }
    }
}
