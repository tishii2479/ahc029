mod def;
mod interactor;
mod simulator;
mod util;

use crate::def::*;
use crate::interactor::*;
use crate::simulator::*;
use crate::util::*;

fn read_status<I: Interactor>(
    state: &mut State,
    input: &Input,
    interactor: &mut I,
) -> Vec<(Card, i64)> {
    let (projects, score, new_cards) = interactor.read_status(input);
    state.projects = projects;
    state.score = score;
    new_cards
}

fn use_card<I: Interactor>(use_card: usize, m: usize, state: &mut State, interactor: &mut I) {
    interactor.output_c(use_card, m);
    if let Card::Invest = state.cards[use_card] {
        state.invest_level += 1;
    }
    state.cards[use_card] = Card::None;
}

fn refill_card<I: Interactor>(
    selected_card: usize,
    new_cards: &Vec<(Card, i64)>,
    state: &mut State,
    interactor: &mut I,
) {
    interactor.output_r(selected_card);
    let i = state.empty_card_index().unwrap();
    state.cards[i] = new_cards[selected_card].0;
}

impl State {
    fn remain_w(&self, t: usize, p: i64) -> i64 {
        (999 - t as i64) * 2_i64.pow(self.invest_level as u32) + (self.score - p)
    }

    fn eval(&self, card: &Card, p: i64, t: usize, refill: bool, montecarlo: bool) -> (f64, usize) {
        if p > self.score {
            return (-INF, 0);
        }

        let overflow_alpha: f64 = if refill { 0.5 } else { 2. };
        let cancel_alpha: f64 = if refill { 1.1 } else { 5. };

        match card {
            Card::WorkSingle(w) => {
                let m = (0..self.projects.len())
                    .max_by_key(|&i| {
                        if self.projects[i].h > w + self.remain_w(t, p) {
                            return -INF as i64 - self.projects[i].h;
                        }
                        (((*w as f64 / self.projects[i].h as f64).min(1.).powf(2.)
                            * self.projects[i].v as f64
                            - ((w - self.projects[i].h).max(0) as f64))
                            * 10000.) as i64
                    })
                    .unwrap();
                if self.projects[m].h > w + self.remain_w(t, p) && p > 0 {
                    return (-INF, m);
                }
                let eval = *w as f64
                    - p as f64
                    - ((w - self.projects[m].h).max(0) as f64) * overflow_alpha;
                (eval, m)
            }
            Card::WorkAll(w) => {
                let mut projects = self.projects.clone();
                projects.sort_by_key(|p| p.h);
                let feasible_proj_count = {
                    let mut c = 0;
                    let mut remain_w = self.remain_w(t, p);
                    for p in projects {
                        if p.h - w <= remain_w {
                            remain_w -= p.h - w; // NOTE: max(0)を取るのが正しいが、取らない方がスコアが良い
                            c += 1;
                        }
                    }
                    c
                };
                let w_sum = (*w * feasible_proj_count) as f64;
                let eval = w_sum
                    - p as f64
                    - (self
                        .projects
                        .iter()
                        .map(|proj| (*w as f64 - proj.h as f64).max(0.) * overflow_alpha)
                        .sum::<f64>());
                (eval, 0)
            }
            Card::CancelSingle => {
                let m = (0..self.projects.len())
                    .max_by_key(|&i| {
                        ((self.projects[i].h as f64 - self.projects[i].v as f64)
                            / self.projects[i].h as f64
                            * 10000.)
                            .round() as i64
                    })
                    .unwrap();
                if t >= self.cancel_limit() {
                    return (-INF, m);
                }
                let eval =
                    self.projects[m].h as f64 * cancel_alpha - self.projects[m].v as f64 - p as f64;
                (eval, m)
            }
            Card::CancelAll => {
                if t >= self.cancel_limit() {
                    return (-INF, 0);
                }
                let eval = self
                    .projects
                    .iter()
                    .map(|proj| proj.h as f64 * cancel_alpha - proj.v as f64)
                    .sum::<f64>()
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
                    || ((t >= self.invest_limit() || self.last_invest_round + 1 == t) && p == 0)
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
                if self.invest_level >= MAX_INVEST_LEVEL || t >= self.invest_limit() {
                    return -INF;
                }
                let invest_card_count = self
                    .cards
                    .iter()
                    .filter(|&&card| card == Card::Invest)
                    .count();
                if (self.score as f64 >= p as f64 * 1.5
                    && p / 2_i64.pow(self.invest_level as u32) < self.invest_cost())
                    || invest_card_count == self.cards.len() - 1
                {
                    INF
                } else {
                    -INF
                }
            }
            _ => self.eval(card, p, t, true, false).0,
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
            .map(|card| self.eval(&card, 0, t, false, true))
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

    fn invest_cost(&self) -> i64 {
        500
    }

    fn invest_limit(&self) -> usize {
        900
        // TODO: モンテカルロで最適なターンを求めた方が良い
        //     let invest_count = self.invest_level
        //         + self
        //             .cards
        //             .iter()
        //             .filter(|&&card| Card::Invest == card)
        //             .count();
        //     let mean_round = self.last_invest_round / invest_count.max(1);
        //     1000 - mean_round.max(100)
    }

    fn cancel_limit(&self) -> usize {
        960
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

fn montecarlo(
    rounds: usize,
    cur_state: &State,
    input: &Input,
    cur_t: usize,
    x: &Vec<i64>,
    refill_first: bool,
    new_select_card: usize,
    new_cards: &Vec<(Card, i64)>,
) -> i64 {
    let mut score_sum = 0;

    for _ in 0..rounds {
        let mut state = cur_state.clone();
        let mut mock_interactor = MockInteractor::new(
            x,
            cur_t,
            &state,
            state.empty_card_index().unwrap_or(0),
            new_cards.clone(),
        );

        // 最初のrefillを固定する場合
        if refill_first {
            refill_card(new_select_card, new_cards, &mut state, &mut mock_interactor);
        }

        for t in cur_t..input.t {
            // 今持っているカードを見て、使うカードを決める
            let (select_card, m) = state.select_use_card(t);

            if state.cards[select_card] == Card::Invest {
                state.last_invest_round = t;
            }
            use_card(select_card, m, &mut state, &mut mock_interactor);
            let new_cards = read_status(&mut state, input, &mut mock_interactor);

            // 新しいカードを見て、補充するカードを決める
            let new_card = if t < input.t - 1 {
                state.select_new_card(&new_cards, t)
            } else {
                0
            };
            refill_card(new_card, &new_cards, &mut state, &mut mock_interactor);
        }
        score_sum += state.score;
    }
    score_sum / rounds as i64
}

fn solve(state: &mut State, input: &Input, interactor: &mut IOInteractor) {
    let mut recorder = Recorder::new();

    for t in 0..input.t {
        // 今持っているカードを見て、使うカードを決める
        let (select_card, m) = state.select_use_card(t);

        if state.cards[select_card] == Card::Invest {
            state.last_invest_round = t;
        }
        use_card(select_card, m, state, interactor);
        recorder.scores.push(state.score);

        let new_cards = read_status(state, input, interactor);
        for (card, _) in new_cards.iter() {
            recorder.x[card.to_t()] += 1;
        }

        // 新しいカードを見て、補充するカードを決める
        let new_card = if t < 980 {
            if t < input.t - 1 {
                state.select_new_card(&new_cards, t)
            } else {
                0
            }
        } else {
            (0..new_cards.len())
                .max_by_key(|&i| {
                    if new_cards[i].0 == Card::Invest && state.invest_level == 20 {
                        return 0;
                    }
                    if new_cards[i].1 <= state.score {
                        montecarlo(100, &state, input, t, &recorder.x, true, i, &new_cards)
                    } else {
                        0
                    }
                })
                .unwrap()
        };
        if new_cards[new_card].0 == Card::Invest {
            recorder.invest_rounds.push(t);
        }
        refill_card(new_card, &new_cards, state, interactor);
    }

    // ビジュアライズ用
    if cfg!(feature = "local") {
        use std::io::Write;
        let mut file = std::fs::File::create("score.log").unwrap();
        writeln!(&mut file, "{:?}", recorder.scores).unwrap();
        writeln!(&mut file, "{:?}", recorder.invest_rounds).unwrap();
    }
}

fn main() {
    time::start_clock();
    let mut interactor = IOInteractor::new();
    let (input, mut state) = interactor.read_input();

    solve(&mut state, &input, &mut interactor);
    eprintln!(
        "result: {{\"score\": {}, \"duration\": {:.4}, \"invest_level\": {}}}",
        state.score,
        time::elapsed_seconds(),
        state.invest_level,
    );
}
