mod def;
mod interactor;
mod simulator;
mod solver;
mod util;

use crate::def::*;
use crate::interactor::*;
use crate::simulator::*;
use crate::solver::*;
use crate::util::*;

fn solve(state: State, input: &Input, interactor: &mut IOInteractor) -> State {
    let mut recorder = Recorder::new();
    let mut solver = Solver { state };

    for t in 0..input.t {
        // 今持っているカードを見て、使うカードを決める
        let (select_card, m) = solver.select_use_card(t);

        if solver.state.cards[select_card] == Card::Invest {
            solver.state.last_invest_round = t;
        }
        solver.state.use_card(select_card, m, interactor);
        recorder.scores.push(solver.state.score);

        let new_cards = solver.state.read_status(input, interactor);
        for (card, _) in new_cards.iter() {
            recorder.x[card.to_t()] += 1;
        }

        // 新しいカードを見て、補充するカードを決める
        let new_card = if t < input.t - 1 {
            solver.select_new_card(&new_cards, t)
        } else {
            0
        };
        if new_cards[new_card].0 == Card::Invest {
            recorder.invest_rounds.push(t);
        }
        solver.state.refill_card(new_card, &new_cards, interactor);
    }

    // ビジュアライズ用
    if cfg!(feature = "local") {
        use std::io::Write;
        let mut file = std::fs::File::create("score.log").unwrap();
        writeln!(&mut file, "{:?}", recorder.scores).unwrap();
        writeln!(&mut file, "{:?}", recorder.invest_rounds).unwrap();
    }

    solver.state
}

fn main() {
    time::start_clock();
    let mut interactor = IOInteractor::new();
    let (input, mut state) = interactor.read_input();

    let state = solve(state, &input, &mut interactor);
    eprintln!(
        "result: {{\"score\": {}, \"duration\": {:.4}, \"invest_level\": {}}}",
        state.score,
        time::elapsed_seconds(),
        state.invest_level,
    );
}
