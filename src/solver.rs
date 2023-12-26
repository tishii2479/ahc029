use crate::def::*;
use crate::interactor::*;
use crate::simulator::*;

pub struct Solver {
    pub state: State,
    pub param: Param,
}

impl Solver {
    pub fn remain_w(&self, t: usize, p: i64) -> i64 {
        (999 - t as i64) * 2_i64.pow(self.state.invest_level as u32) + (self.state.score - p)
    }

    pub fn solve(&mut self, input: &Input, interactor: &mut IOInteractor) {
        let mut recorder = Recorder::new();

        for t in 0..input.t {
            // 今持っているカードを見て、使うカードを決める
            let (select_card, m) = self.select_use_card(t);

            if self.state.cards[select_card] == Card::Invest {
                self.state.last_invest_round = t;
            }
            self.state.use_card(select_card, m, interactor);
            recorder.scores.push(self.state.score);

            let new_cards = self.state.read_status(input, interactor);
            for (card, _) in new_cards.iter() {
                recorder.x[card.to_t()] += 1;
            }

            // 新しいカードを見て、補充するカードを決める
            const MONTE_CARLO_ROUND: usize = 30;
            let new_card = if t < 990 {
                self.select_new_card(&new_cards, t)
            } else if t < input.t - 1 {
                (0..new_cards.len())
                    .max_by_key(|&i| {
                        if new_cards[i].1 <= self.state.score {
                            montecarlo(
                                MONTE_CARLO_ROUND,
                                &self.state,
                                &self.param,
                                input,
                                t,
                                &recorder.x,
                                true,
                                i,
                                &new_cards,
                            )
                        } else {
                            -1
                        }
                    })
                    .unwrap()
            } else {
                0
            };
            // モンテカルロしない場合
            // let new_card = if t < input.t - 1 {
            //     self.select_new_card(&new_cards, t)
            // } else {
            //     0
            // };
            if new_cards[new_card].0 == Card::Invest {
                recorder.invest_rounds.push(t);
            }
            self.state.refill_card(new_card, &new_cards, interactor);
        }

        // ビジュアライズ用
        if cfg!(feature = "local") {
            use std::io::Write;
            let mut file = std::fs::File::create("score.log").unwrap();
            writeln!(&mut file, "{:?}", recorder.scores).unwrap();
            writeln!(&mut file, "{:?}", recorder.invest_rounds).unwrap();
        }
    }

    pub fn eval(&self, card: &Card, p: i64, t: usize, refill: bool) -> (f64, usize) {
        if p > self.state.score {
            return (-INF, 0);
        }

        let overflow_alpha = if refill {
            self.param.overflow_alpha_refill
        } else {
            self.param.overflow_alpha
        };
        let overflow_alpha_all = if refill {
            self.param.overflow_alpha_all_refill
        } else {
            self.param.overflow_alpha_all
        };
        let cancel_alpha = if refill {
            self.param.cancel_alpha_refill
        } else {
            self.param.cancel_alpha
        };
        let cancel_alpha_all = if refill {
            self.param.cancel_alpha_all_refill
        } else {
            self.param.cancel_alpha_all
        };

        match card {
            Card::WorkSingle(w) => {
                let m = (0..self.state.projects.len())
                    .max_by_key(|&i| {
                        if self.state.projects[i].h > w + self.remain_w(t, p) {
                            return -INF as i64 - self.state.projects[i].h;
                        }
                        (((*w as f64 / self.state.projects[i].h as f64)
                            .min(1.)
                            .powf(self.param.work_single_beta)
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
                        .map(|proj| (*w as f64 - proj.h as f64).max(0.) * overflow_alpha_all)
                        .sum::<f64>());
                (eval, 0)
            }
            Card::CancelSingle => {
                let m = (0..self.state.projects.len())
                    .max_by_key(|&i| {
                        ((self.state.projects[i].h as f64 - self.state.projects[i].v as f64)
                            / self.state.projects[i].h as f64 // TODO: 消す？
                            * 10000.)
                            .round() as i64
                    })
                    .unwrap();
                if t >= self.param.cancel_limit {
                    return (-INF, m);
                }
                let eval = self.state.projects[m].h as f64 * cancel_alpha
                    - self.state.projects[m].v as f64
                    - p as f64;
                (eval, m)
            }
            Card::CancelAll => {
                if t >= self.param.cancel_limit {
                    return (-INF, 0);
                }
                let eval = self
                    .state
                    .projects
                    .iter()
                    .map(|proj| proj.h as f64 * cancel_alpha_all - proj.v as f64)
                    .sum::<f64>()
                    - p as f64;
                (eval, 0)
            }
            Card::Invest => {
                if self.state.invest_level >= MAX_INVEST_LEVEL {
                    return (-INF, 0);
                }
                let invest_card_count = self
                    .state
                    .cards
                    .iter()
                    .filter(|&&card| card == Card::Invest)
                    .count();
                // 1. 手持ちのカードが全て増資になった場合
                // 2. 増資の期限が来た場合
                // 3. 前回増資した場合（増資カードを消費している場合）
                // 4. 増資回数がMAX_INVEST_LEVELに到達する場合
                if self.state.cards.len() == invest_card_count
                    || ((t >= self.param.invest_limit || self.state.last_invest_round + 1 == t)
                        && p == 0)
                    || invest_card_count + self.state.invest_level == MAX_INVEST_LEVEL
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
                if self.state.invest_level >= MAX_INVEST_LEVEL || t >= self.param.invest_limit {
                    return -INF;
                }
                let invest_card_count = self
                    .state
                    .cards
                    .iter()
                    .filter(|&&card| card == Card::Invest)
                    .count();
                if (self.state.score as f64 >= p as f64 * 1.5
                    && p / 2_i64.pow(self.state.invest_level as u32) < self.param.invest_cost)
                    || invest_card_count == self.state.cards.len() - 1
                    || self.state.invest_level == MAX_INVEST_LEVEL - 1
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
}
