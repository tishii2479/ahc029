use rand::{prelude::*, Rng};
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use rand_distr::{Normal, WeightedIndex};

use crate::{def::*, interactor::Interactor, solver::*, util::rnd};

fn generate_project(rng: &mut ChaCha20Rng) -> Project {
    let b = rng.gen_range(2.0f64..=8.0);
    let h = 2.0f64.powf(b).round() as i64;
    let normal_dist = Normal::<f64>::new(b, 0.5).unwrap();
    let v = 2.0f64
        .powf(normal_dist.sample(rng).clamp(0.0, 10.0))
        .round() as i64;
    Project { h, v }
}

fn generate_card(rng: &mut ChaCha20Rng, m: usize, x: &Vec<i64>) -> (Card, i64) {
    let weighted_index = WeightedIndex::new(x).unwrap();
    let t = weighted_index.sample(rng);
    let mut w = 0;
    let mut p;
    match t {
        0 => {
            w = rng.gen_range(1i64..=50);
            let mu = w as f64;
            let normal_dist = Normal::<f64>::new(mu, mu / 3.0).unwrap();
            p = normal_dist.sample(rng).round() as i64;
            p = p.clamp(1, 10000);
        }
        1 => {
            w = rng.gen_range(1i64..=50);
            let mu = w as f64 * m as f64;
            let normal_dist = Normal::<f64>::new(mu, mu / 3.0).unwrap();
            p = normal_dist.sample(rng).round() as i64;
            p = p.clamp(1, 10000);
        }
        2 => p = rng.gen_range(0i64..=10),
        3 => p = rng.gen_range(0i64..=10),
        4 => p = rng.gen_range(200i64..=1000),
        _ => panic!(),
    }

    (Card::from_tw(t, w), p)
}

pub struct MockInteractor {
    t: usize,
    score: i64,
    invest_level: usize,
    cards: Vec<Card>,
    projects: Vec<Project>,
    used_card: usize,
    candidate_cards: Vec<(Card, i64)>,
    new_projects: Vec<Project>,
    new_cards: Vec<Vec<(Card, i64)>>,
}

impl MockInteractor {
    pub fn new(
        x: &Vec<i64>,
        t: usize,
        state: &State,
        used_card: usize,
        candidate_cards: Vec<(Card, i64)>,
    ) -> MockInteractor {
        let mut rng = ChaCha20Rng::seed_from_u64(rnd::gen_range(0, 10000000000) as u64);
        let mut new_projects = vec![];
        for _ in 0..state.projects.len() * (1005 - t) {
            new_projects.push(generate_project(&mut rng));
        }
        let mut new_cards = vec![];
        for i in 0..(1005 - t) {
            new_cards.push(vec![]);
            new_cards[i].push((Card::WorkSingle(1), 0));
            for _ in 1..state.cards.len() {
                new_cards[i].push(generate_card(&mut rng, state.projects.len(), &x));
            }
        }

        MockInteractor {
            t: 0,
            score: state.score,
            invest_level: state.invest_level,
            cards: state.cards.clone(),
            projects: state.projects.clone(),
            used_card,
            candidate_cards,
            new_projects,
            new_cards,
        }
    }

    fn get_card_candidate(&mut self) -> Vec<(Card, i64)> {
        let mut cards = self.new_cards[self.t].clone();
        for card in cards.iter_mut() {
            match &mut card.0 {
                Card::WorkSingle(w) => *w *= 1 << self.invest_level,
                Card::WorkAll(w) => *w *= 1 << self.invest_level,
                _ => {}
            }
            card.1 *= 1 << self.invest_level;
        }
        self.t += 1;
        cards
    }
    fn update_project(&mut self, m: usize) {
        let project = self.new_projects.pop().unwrap();
        self.projects[m] = Project {
            h: project.h * (1 << self.invest_level),
            v: project.v * (1 << self.invest_level),
        };
    }

    fn work_project(&mut self, m: usize, w: i64) {
        self.projects[m].h -= w;
        if self.projects[m].h <= 0 {
            self.score += self.projects[m].v;
            self.update_project(m);
        }
    }
}

impl Interactor for MockInteractor {
    fn output_c(&mut self, c: usize, m: usize) {
        match self.cards[c] {
            Card::WorkSingle(w) => {
                self.work_project(m, w);
            }
            Card::WorkAll(w) => {
                for i in 0..self.projects.len() {
                    self.work_project(i, w);
                }
            }
            Card::CancelSingle => self.update_project(m),
            Card::CancelAll => {
                for i in 0..self.projects.len() {
                    self.update_project(i);
                }
            }
            Card::Invest => self.invest_level += 1,
            _ => panic!(),
        }
        self.used_card = c;
    }

    fn output_r(&mut self, r: usize) {
        self.score -= self.candidate_cards[r].1;
        self.cards[self.used_card] = self.candidate_cards[r].0;
    }

    #[allow(unused)]
    fn read_status(
        &mut self,
        input: &Input,
    ) -> (Vec<crate::def::Project>, i64, Vec<(crate::def::Card, i64)>) {
        self.candidate_cards = self.get_card_candidate();
        (
            self.projects.clone(),
            self.score,
            self.candidate_cards.clone(),
        )
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
        let mut solver = Solver {
            state: cur_state.clone(),
        };
        let mut mock_interactor = MockInteractor::new(
            x,
            cur_t,
            &solver.state,
            solver.state.empty_card_index().unwrap_or(0),
            new_cards.clone(),
        );

        // 最初のrefillを固定する場合
        if refill_first {
            solver
                .state
                .refill_card(new_select_card, new_cards, &mut mock_interactor);
        }

        for t in cur_t..input.t {
            // 今持っているカードを見て、使うカードを決める
            let (select_card, m) = solver.select_use_card(t);

            if solver.state.cards[select_card] == Card::Invest {
                solver.state.last_invest_round = t;
            }
            solver.state.use_card(select_card, m, &mut mock_interactor);
            let new_cards = solver.state.read_status(input, &mut mock_interactor);

            // 新しいカードを見て、補充するカードを決める
            let new_card = if t < input.t - 1 {
                solver.select_new_card(&new_cards, t)
            } else {
                0
            };
            solver
                .state
                .refill_card(new_card, &new_cards, &mut mock_interactor);
        }
        score_sum += solver.state.score;
    }
    score_sum / rounds as i64
}
