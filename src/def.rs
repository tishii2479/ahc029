pub const MAX_INVEST_LEVEL: usize = 20;
pub const INF: f64 = 1e18;

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

#[derive(Debug, Clone)]
pub struct State {
    pub last_invest_round: usize,
    pub invest_level: usize,
    pub score: i64,
    pub cards: Vec<Card>,
    pub projects: Vec<Project>,
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
