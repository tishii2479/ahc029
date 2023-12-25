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

const CANCEL_LIMIT: usize = 980;

impl State {
    fn is_feasible(&self, project: &Project, p: i64, w: i64, t: usize) -> bool {
        project.h
            <= w + (999 - t as i64) * 2_i64.pow(self.invest_level as u32)
                + (self.score - p) * 2 / 10
    }

    fn eval(&self, card: &Card, p: i64, t: usize) -> (f64, usize) {
        if p > self.score {
            return (-INF, 0);
        }
        match card {
            Card::WorkSingle(w) => {
                let m = (0..self.projects.len())
                    .max_by_key(|&i| {
                        if !self.is_feasible(&self.projects[i], p, *w, t) {
                            return -INF as i64;
                        }
                        (((*w as f64 / self.projects[i].h as f64).min(1.).powf(2.)
                            * self.projects[i].v as f64
                            - ((w - self.projects[i].h).max(0) as f64))
                            * 10000.) as i64
                    })
                    .unwrap();
                if !self.is_feasible(&self.projects[m], p, *w, t) && p > 0 {
                    return (-INF, m);
                }
                let eval = *w as f64 - p as f64 - ((w - self.projects[m].h).max(0) as f64);
                (eval, m)
            }
            Card::WorkAll(w) => {
                let feasible_projects: Vec<Project> = self
                    .projects
                    .iter()
                    .copied()
                    .filter(|proj| self.is_feasible(proj, p, *w, t))
                    .collect();
                let w_sum = *w as f64 * feasible_projects.len() as f64;
                let eval = w_sum
                    - p as f64
                    - (w_sum - feasible_projects.iter().map(|proj| proj.h).sum::<i64>() as f64)
                        .max(0.);
                (eval, 0)
            }
            Card::CancelSingle => {
                let m = (0..self.projects.len())
                    .max_by_key(|&i| self.projects[i].h - self.projects[i].v)
                    .unwrap();
                if t >= CANCEL_LIMIT {
                    return (-INF, 0);
                }
                let eval = (&self.projects[m].h - self.projects[m].v - p) as f64;
                (eval, m)
            }
            Card::CancelAll => {
                if t >= CANCEL_LIMIT {
                    return (-INF, 0);
                }
                let eval = self
                    .projects
                    .iter()
                    .map(|proj| proj.h - proj.v)
                    .sum::<i64>() as f64
                    - p as f64;
                (eval, 0)
            }
            Card::Invest => {
                if self.invest_level >= MAX_INVEST_LEVEL {
                    return (-INF, 0);
                }
                if self.cards.len()
                    == self
                        .cards
                        .iter()
                        .filter(|&&card| card == Card::Invest)
                        .count()
                    || ((t >= self.should_invest_limit() || self.last_invest_round + 1 == t)
                        && p == 0)
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
                if self.invest_level >= MAX_INVEST_LEVEL || t >= self.should_invest_limit() {
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

    fn should_invest_limit(&self) -> usize {
        850
        // let invest_count = self.invest_level
        //     + self
        //         .cards
        //         .iter()
        //         .filter(|&&card| Card::Invest == card)
        //         .count();
        // let mean_round = self.last_invest_round / invest_count.max(1);
        // 1000 - mean_round * 3
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
