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
        fn eval_work(project: &Project, w: i64, t: usize, invest_level: usize) -> f64 {
            if project.h > (1000 - t as i64) * 2_i64.pow(invest_level as u32) {
                return 0.;
            }
            (w as f64 / project.h as f64).min(1.).powf(2.) * project.v as f64
                - ((w - project.h).max(0) as f64).powf(0.8)
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
                    .max_by_key(|&i| {
                        (eval_work(&self.projects[i], *w, t, self.invest_level) * 10000.) as i64
                    })
                    .unwrap();
                let eval =
                    *w as f64 - p as f64 - ((w - self.projects[m].h).max(0) as f64).powf(0.8);
                (eval, m)
            }
            Card::WorkAll(w) => {
                // let eval = self
                //     .projects
                //     .iter()
                //     .map(|proj| eval_work(&proj, *w, t, self.invest_level))
                //     .sum::<f64>()
                //     - p as f64;
                let w_sum = *w as f64 * self.projects.len() as f64;
                let eval = w_sum
                    - p as f64
                    - (w_sum - self.projects.iter().map(|&proj| proj.h).sum::<i64>() as f64)
                        .max(0.)
                        .powf(0.8);
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
                if self.cards.len()
                    == self
                        .cards
                        .iter()
                        .filter(|&&card| card == Card::Invest)
                        .count()
                    || t >= 800
                    || self.last_invest_round + 1 == t
                {
                    return (INF, 0);
                }
                (-INF, 0)
            }
            Card::None => (-INF, 0),
        }
    }

    fn eval_refill(&self, card: &Card, p: i64, t: usize) -> f64 {
        if p > self.score {
            return -INF;
        }

        match card {
            Card::Invest => {
                if self.invest_level >= MAX_INVEST_LEVEL || !self.should_invest(t) {
                    return -INF;
                }
                if self.score >= p && p / 2_i64.pow(self.invest_level as u32) < 600 {
                    INF
                } else {
                    -INF
                }
            }
            _ => self.eval(card, p, t).0,
        }
    }

    fn select_new_card(&self, new_cards: &Vec<(Card, i64)>, t: usize) -> usize {
        let eval_refills: Vec<f64> = new_cards
            .iter()
            .map(|(card, p)| self.eval_refill(&card, *p, t))
            .collect();
        let mut card_idx = (0..new_cards.len()).collect::<Vec<usize>>();
        card_idx.sort_by(|i, j| eval_refills[*j].partial_cmp(&eval_refills[*i]).unwrap());

        println!("# eval_refill, m, card_type, p");
        for &i in card_idx.iter() {
            println!(
                "# {} {:.3} {:?}, {}",
                i, eval_refills[i], new_cards[i].0, new_cards[i].1
            );
        }

        card_idx[0]
    }

    fn select_use_card(&self, t: usize) -> (usize, usize) {
        let evals: Vec<(f64, usize)> = self
            .cards
            .iter()
            .map(|card| self.eval(&card, 0, t))
            .collect();

        let mut card_idx = (0..self.cards.len()).collect::<Vec<usize>>();
        card_idx.sort_by(|i, j| evals[*j].partial_cmp(&evals[*i]).unwrap());

        println!("# eval, m, card_type, p");
        for &i in card_idx.iter() {
            println!(
                "# {} {:.3} {} {:?}",
                i, evals[i].0, evals[i].1, self.cards[i]
            );
        }

        (card_idx[0], evals[card_idx[0]].1)
    }

    fn should_invest(&self, t: usize) -> bool {
        t < 810
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

fn solve(state: &mut State, input: &Input, interactor: &mut Interactor) {
    let mut scores = vec![0];
    let mut invest_rounds = vec![];

    for t in 0..input.t {
        // 今持っているカードを見て、使うカードを決める
        let (select_card, m) = state.select_use_card(t);

        if state.cards[select_card] == Card::Invest {
            state.last_invest_round = t;
            invest_rounds.push(t);
        }
        use_card(select_card, m, state, interactor);
        scores.push(state.score);

        let new_cards = read_status(state, input, interactor);

        // 新しいカードを見て、補充するカードを決める
        let new_card = if t < input.t - 1 {
            state.select_new_card(&new_cards, t)
        } else {
            0
        };
        refill_card(new_card, &new_cards, state, interactor);
    }

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
