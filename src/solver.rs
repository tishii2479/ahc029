use crate::def::*;

pub struct Solver {
    pub state: State,
}

impl Solver {
    pub fn remain_w(&self, t: usize, p: i64) -> i64 {
        (999 - t as i64) * 2_i64.pow(self.state.invest_level as u32) + (self.state.score - p)
    }

    pub fn eval(&self, card: &Card, p: i64, t: usize, refill: bool) -> (f64, usize) {
        if p > self.state.score {
            return (-INF, 0);
        }

        let overflow_alpha: f64 = if refill { 0.5 } else { 2. };
        let cancel_alpha: f64 = if refill { 1.1 } else { 5. };

        match card {
            Card::WorkSingle(w) => {
                let m = (0..self.state.projects.len())
                    .max_by_key(|&i| {
                        if self.state.projects[i].h > w + self.remain_w(t, p) {
                            return -INF as i64 - self.state.projects[i].h;
                        }
                        (((*w as f64 / self.state.projects[i].h as f64)
                            .min(1.)
                            .powf(2.)
                            * self.state.projects[i].v as f64
                            - ((w - self.state.projects[i].h).max(0) as f64))
                            * 10000.) as i64
                    })
                    .unwrap();
                if self.state.projects[m].h > w + self.remain_w(t, p) && p > 0 {
                    return (-INF, m);
                }
                let eval = *w as f64
                    - p as f64
                    - ((w - self.state.projects[m].h).max(0) as f64) * overflow_alpha;
                (eval, m)
            }
            Card::WorkAll(w) => {
                let mut projects = self.state.projects.clone();
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
                        .state
                        .projects
                        .iter()
                        .map(|proj| (*w as f64 - proj.h as f64).max(0.) * overflow_alpha)
                        .sum::<f64>());
                (eval, 0)
            }
            Card::CancelSingle => {
                let m = (0..self.state.projects.len())
                    .max_by_key(|&i| {
                        ((self.state.projects[i].h as f64 - self.state.projects[i].v as f64)
                            / self.state.projects[i].h as f64
                            * 10000.)
                            .round() as i64
                    })
                    .unwrap();
                if t >= self.cancel_limit() {
                    return (-INF, m);
                }
                let eval = self.state.projects[m].h as f64 * cancel_alpha
                    - self.state.projects[m].v as f64
                    - p as f64;
                (eval, m)
            }
            Card::CancelAll => {
                if t >= self.cancel_limit() {
                    return (-INF, 0);
                }
                let eval = self
                    .state
                    .projects
                    .iter()
                    .map(|proj| proj.h as f64 * cancel_alpha - proj.v as f64)
                    .sum::<f64>()
                    - p as f64;
                (eval, 0)
            }
            Card::Invest => {
                if self.state.invest_level >= MAX_INVEST_LEVEL {
                    return (-INF, 0);
                }
                if self.state.cards.len()
                    == self
                        .state
                        .cards
                        .iter()
                        .filter(|&&card| card == Card::Invest)
                        .count()
                    || ((t >= self.invest_limit() || self.state.last_invest_round + 1 == t)
                        && p == 0)
                {
                    return (INF, 0);
                }
                (-INF, 0)
            }
            Card::None => (-INF, 0),
        }
    }

    pub fn eval_refill(&self, card: &Card, p: i64, t: usize) -> f64 {
        if p > self.state.score {
            return -INF;
        }

        match card {
            Card::Invest => {
                if self.state.invest_level >= MAX_INVEST_LEVEL || t >= self.invest_limit() {
                    return -INF;
                }
                let invest_card_count = self
                    .state
                    .cards
                    .iter()
                    .filter(|&&card| card == Card::Invest)
                    .count();
                if (self.state.score as f64 >= p as f64 * 1.5
                    && p / 2_i64.pow(self.state.invest_level as u32) < self.invest_cost())
                    || invest_card_count == self.state.cards.len() - 1
                {
                    INF
                } else {
                    -INF
                }
            }
            _ => self.eval(card, p, t, true).0,
        }
    }

    pub fn select_new_card(&self, new_cards: &Vec<(Card, i64)>, t: usize) -> usize {
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

    pub fn select_use_card(&self, t: usize) -> (usize, usize) {
        let evals: Vec<(f64, usize)> = self
            .state
            .cards
            .iter()
            .map(|card| self.eval(&card, 0, t, false))
            .collect();

        let mut card_idx = (0..self.state.cards.len()).collect::<Vec<usize>>();
        card_idx.sort_by(|i, j| evals[*j].partial_cmp(&evals[*i]).unwrap());

        println!("# eval, m, card_type, p");
        for &i in card_idx.iter() {
            println!(
                "# {} {:.3} {} {:?}",
                i, evals[i].0, evals[i].1, self.state.cards[i]
            );
        }

        (card_idx[0], evals[card_idx[0]].1)
    }

    pub fn invest_cost(&self) -> i64 {
        500
    }

    pub fn invest_limit(&self) -> usize {
        900
        // TODO: モンテカルロで最適なターンを求めた方が良い
        //     let invest_count = self.state.invest_level
        //         + self
        //             .cards
        //             .iter()
        //             .filter(|&&card| Card::Invest == card)
        //             .count();
        //     let mean_round = self.state.last_invest_round / invest_count.max(1);
        //     1000 - mean_round.max(100)
    }

    pub fn cancel_limit(&self) -> usize {
        960
    }
}
