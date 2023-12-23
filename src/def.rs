pub const MAX_INVEST_LEVEL: usize = 20;
pub const INF: f64 = 1e30;

#[derive(Debug, Clone, Copy)]
pub enum Card {
    WorkSingle(i64),
    WorkAll(i64),
    CancelSingle,
    CancelAll,
    Invest,
    None,
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
}

pub struct Input {
    pub n: usize,
    pub m: usize,
    pub k: usize,
    pub t: usize,
}

#[derive(Debug)]
pub struct State {
    pub invest_level: usize,
    pub score: i64,
    /// t, w
    pub cards: Vec<Card>,
    /// h, v
    pub projects: Vec<(i64, i64)>,
}
