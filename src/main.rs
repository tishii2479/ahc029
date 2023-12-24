mod def;
mod interactor;
mod util;

use crate::def::*;
use crate::interactor::*;
use crate::util::*;

fn read_status(state: &mut State, input: &Input, interactor: &mut Interactor) -> Vec<(Card, i64)> {
    let (projects, score, new_cards) = interactor.read_status(input);
    state.projects = projects;
    state.score = score;
    new_cards
}

fn use_card(use_card: usize, m: usize, state: &mut State, interactor: &mut Interactor) {
    interactor.output_c(use_card, m);
    if let Card::Invest = state.cards[use_card] {
        state.invest_level += 1;
    }
    state.cards[use_card] = Card::None;
}

fn refill_card(
    selected_card: usize,
    new_cards: &Vec<(Card, i64)>,
    state: &mut State,
    interactor: &mut Interactor,
) {
    interactor.output_r(selected_card);
    let i = state.empty_card_index().unwrap();
    state.cards[i] = new_cards[selected_card].0;
}

impl State {
    fn eval(&self, card: &Card, p: i64, t: usize) -> (f64, usize) {
        fn eval_work(project: (i64, i64), w: i64) -> f64 {
            // ISSUE: ペナルティはない・もっと小さい方が良いかも
            const GAMMA: f64 = 0.8;
            (w as f64 / project.0 as f64).min(1.).powf(2.) * project.1 as f64
                - project.1 as f64 * ((w - project.0).max(0) as f64).powf(GAMMA) / project.0 as f64
        }

        fn eval_cancel(project: (i64, i64)) -> f64 {
            (project.0 - project.1) as f64
        }

        if p > self.score {
            return (-INF, 0);
        }

        let b = if t < 800 {
            1.
        } else {
            1. - ((t as f64 - 800.) / 100.).sqrt()
        }
        .clamp(0., 1.);

        match card {
            Card::WorkSingle(w) => {
                let m = (0..self.projects.len())
                    .min_by_key(|&i| self.projects[i].0)
                    .unwrap();
                let eval = eval_work(self.projects[m], *w) * b - p as f64;
                (eval, m)
            }
            Card::WorkAll(w) => {
                let eval = self
                    .projects
                    .iter()
                    .map(|proj| eval_work(*proj, *w))
                    .sum::<f64>()
                    * b
                    - p as f64;
                (eval, 0)
            }
            Card::CancelSingle => {
                let m = (0..self.projects.len())
                    .max_by_key(|&i| (eval_cancel(self.projects[i]) * 10000.) as i64)
                    .unwrap();
                let eval = eval_cancel(self.projects[m]) * b - p as f64;
                (eval, m)
            }
            Card::CancelAll => {
                let eval = self
                    .projects
                    .iter()
                    .map(|proj| eval_cancel(*proj))
                    .sum::<f64>()
                    * b
                    - p as f64;
                (eval, 0)
            }
            Card::Invest => {
                if self.invest_level >= MAX_INVEST_LEVEL {
                    return (-INF, 0);
                }
                let eval = if self.score >= p { INF * b } else { -INF };
                (eval, 0)
            }
            Card::None => (-INF, 0),
        }
    }

    fn empty_card_index(&self) -> Option<usize> {
        for i in 0..self.cards.len() {
            if let Card::None = self.cards[i] {
                return Some(i);
            }
        }
        None
    }
}

fn select_best_card(
    state: &mut State,
    new_cards: &Vec<(Card, i64)>,
    t: usize,
) -> (usize, usize, Option<usize>) {
    let mut cards: Vec<(Card, i64)> = state.cards.iter().copied().map(|card| (card, 0)).collect();
    cards.extend(new_cards);
    let evals = cards
        .iter()
        .map(|(card, p)| state.eval(card, *p, t))
        .collect::<Vec<(f64, usize)>>();
    let mut card_idx = (0..cards.len()).collect::<Vec<usize>>();
    card_idx.sort_by(|i, j| evals[*j].partial_cmp(&evals[*i]).unwrap());

    let mut refilled_card = None;
    let mut selected_card = None;
    let mut selected_m = 0;

    println!("# eval, m, card_type, p");
    for i in card_idx {
        println!(
            "# {:.3}, {}, {:?}, {}",
            evals[i].0, evals[i].1, cards[i].0, cards[i].1
        );

        let is_exisiting_card = i < state.cards.len();
        if is_exisiting_card {
            if selected_card.is_none() {
                selected_card = Some(i);
                selected_m = evals[i].1;
            }
        } else {
            if selected_card.is_none() {
                // 前使われた場所を指定する
                selected_card = state.empty_card_index();
                selected_m = evals[i].1;
            }
            if refilled_card.is_none() {
                refilled_card = Some(i - state.cards.len());
            }
        }
    }

    (selected_card.unwrap(), selected_m, refilled_card)
}

fn solve(state: &mut State, input: &Input, interactor: &mut Interactor) {
    let mut scores = vec![0];
    let mut invest_rounds = vec![];

    // 最初のカードを出す
    let (select_card, m, _) = select_best_card(state, &vec![], 0);
    use_card(select_card, m, state, interactor);

    for t in 1..input.t {
        let new_cards = read_status(state, input, interactor);

        // 今持っているカードと新しいカードを見て、使うカード&補充するカードを決める
        let (select_card, m, refilled_card) = select_best_card(state, &new_cards, t);
        refill_card(refilled_card.unwrap(), &new_cards, state, interactor);

        if state.cards[select_card] == Card::Invest {
            invest_rounds.push(t);
        }

        use_card(select_card, m, state, interactor);
        scores.push(state.score);
    }

    // 最後にカードを補充する
    let new_cards = read_status(state, input, interactor);
    refill_card(0, &new_cards, state, interactor);

    // ビジュアライズ用
    if cfg!(feature = "local") {
        use std::io::Write;
        let mut file = std::fs::File::create("score.log").unwrap();
        writeln!(&mut file, "{:?}", scores).unwrap();
        writeln!(&mut file, "{:?}", invest_rounds).unwrap();
    }

    eprintln!("invest_level:    {}", state.invest_level);
}

fn main() {
    time::start_clock();
    let mut interactor = Interactor::new();
    let (input, mut state) = interactor.read_input();

    solve(&mut state, &input, &mut interactor);
    eprintln!(
        "result: {{\"score\": {}, \"duration\": {:.4}}}",
        state.score,
        time::elapsed_seconds()
    );
}
