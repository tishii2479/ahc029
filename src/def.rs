pub enum CardType {
    SINGLE_LABOR,
    ALL_LABOR,
    SINGLE_CANCEL,
    ALL_CANCEL,
    CAPITIAL_INCREASE,
    NONE,
}

impl CardType {
    pub fn from_usize(n: usize) -> CardType {
        match n {
            0 => CardType::SINGLE_LABOR,
            1 => CardType::ALL_LABOR,
            2 => CardType::SINGLE_CANCEL,
            3 => CardType::ALL_CANCEL,
            4 => CardType::CAPITIAL_INCREASE,
            5 => CardType::NONE,
            _ => panic!("invalid card type: {n}"),
        }
    }
}

pub struct Input {
    pub n: usize,
    pub m: usize,
    pub k: usize,
    pub t: usize,
}

pub struct State {
    pub score: i64,
    pub cards: Vec<(CardType, i64)>,
    pub projects: Vec<(i64, i64)>,
}
