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
        fn eval_work(project: &Project, w: i64, alpha: f64, gamma: f64) -> f64 {
            (w as f64 / project.h as f64).min(1.).powf(alpha) * project.v as f64
                - ((w - project.h).max(0) as f64).powf(gamma)
        }

        fn eval_cancel(project: &Project) -> f64 {
            // TODO: hの大きさも考慮する
            project.h as f64 - project.v as f64
        }

        if p > self.score {
            return (-INF, 0);
        }
        match card {
            Card::WorkSingle(w) => {
                let m = (0..self.projects.len())
                    .max_by_key(|&i| (eval_work(&self.projects[i], *w, 2., 0.8) * 10000.) as i64)
                    .unwrap();
                let eval =
                    *w as f64 - p as f64 - ((w - self.projects[m].h).max(0) as f64).powf(0.8);
                (eval, m)
            }
            Card::WorkAll(w) => {
                let eval = self
                    .projects
                    .iter()
                    .map(|proj| eval_work(&proj, *w, 2., 0.5))
                    .sum::<f64>()
                    - p as f64;
                (eval, 0)
            }
            Card::CancelSingle => {
                let m = (0..self.projects.len())
                    .max_by_key(|&i| (eval_cancel(&self.projects[i]) * 10000.) as i64)
                    .unwrap();
                let eval = eval_cancel(&self.projects[m]) - p as f64;
                (eval, m)
            }
            Card::CancelAll => {
                let eval = self
                    .projects
                    .iter()
                    .map(|proj| eval_cancel(proj))
                    .sum::<f64>()
                    - p as f64;
                (eval, 0)
            }
            Card::Invest => {
                if self.invest_level >= MAX_INVEST_LEVEL || !self.should_invest(t) {
                    return (-INF, 0);
                }
                let eval = if self.score >= p { INF } else { -INF };
                (eval, 0)
            }
            Card::None => (-INF, 0),
        }
    }

    fn eval_refill(&self, card: &Card, p: i64, t: usize) -> f64 {
        if p > self.score {
            return -INF;
        }

        let b = if t < 900 {
            1.
        } else {
            1. - ((t as f64 - 900 as f64) / 100.).sqrt()
        }
        .clamp(0., 1.);

        match card {
            Card::WorkSingle(w) => *w as f64 * b - p as f64,
            Card::WorkAll(w) => *w as f64 * self.projects.len() as f64 * b - p as f64,
            Card::CancelSingle => (2_f64).powf(self.invest_level as f64) - p as f64,
            Card::CancelAll => -INF,
            Card::Invest => {
                if self.invest_level >= MAX_INVEST_LEVEL || !self.should_invest(t) {
                    return -INF;
                }
                if self.score >= p {
                    INF
                } else {
                    -INF
                }
            }
            Card::None => -INF,
        }
    }

    fn should_invest(&self, t: usize) -> bool {
        let mean_round = self.last_invest_round as f64 / self.invest_level.max(1) as f64;
        let remain_round = (1000 - t) as f64;
        remain_round >= mean_round
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
        .enumerate()
        .map(|(i, (card, p))| {
            let (mut eval, m) = state.eval(card, *p, t);
            if i > state.cards.len() {
                eval -= 1e-10;
            }
            (eval, m)
        })
        .collect::<Vec<(f64, usize)>>();
    let mut selected_card = (0..cards.len())
        .max_by(|i, j| evals[*i].0.partial_cmp(&evals[*j].0).unwrap())
        .unwrap();

    {
        println!("# eval, m, card_type, p");
        let mut card_idx = (0..cards.len()).collect::<Vec<usize>>();
        card_idx.sort_by(|i, j| evals[*j].partial_cmp(&evals[*i]).unwrap());
        for i in card_idx {
            println!(
                "# {} {:.3}, {}, {:?}, {}",
                i, evals[i].0, evals[i].1, cards[i].0, cards[i].1
            );
        }
    }

    let (selected_m, refill_card) = if new_cards.len() == 0 {
        (evals[selected_card].1, None)
    } else if selected_card < state.cards.len() {
        let eval_refills = new_cards
            .iter()
            .map(|(card, p)| state.eval_refill(card, *p, t))
            .collect::<Vec<f64>>();
        {
            println!("# eval_refill, m, card_type, p");
            let mut card_idx = (0..new_cards.len()).collect::<Vec<usize>>();
            card_idx.sort_by(|i, j| eval_refills[*j].partial_cmp(&eval_refills[*i]).unwrap());
            for i in card_idx {
                println!(
                    "# {} {:.3}, {:?}, {}",
                    i, eval_refills[i], cards[i].0, cards[i].1
                );
            }
        }
        (
            evals[selected_card].1,
            (0..new_cards.len())
                .max_by(|i, j| eval_refills[*i].partial_cmp(&eval_refills[*j]).unwrap()),
        )
    } else {
        let m = evals[selected_card].1;
        let i = selected_card - state.cards.len();
        selected_card = state.empty_card_index().unwrap();
        (m, Some(i))
    };

    (selected_card, selected_m, refill_card)
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
            state.last_invest_round = t;
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
        "result: {{\"score\": {}, \"duration\": {:.4}, \"invest_level\": {}}}",
        state.score,
        time::elapsed_seconds(),
        state.invest_level,
    );
}
