mod def;
mod interactor;
// mod simulator;
mod solver;
mod util;

use crate::interactor::*;
// use crate::simulator::*;
use crate::def::*;
use crate::solver::*;
use crate::util::*;

fn load_params() -> Param {
    if true {
        use std::env;
        let args: Vec<String> = env::args().collect();
        Param {
            overflow_alpha_refill: args[1].parse().unwrap(),
            overflow_alpha: args[2].parse().unwrap(),
            cancel_alpha_refill: args[3].parse().unwrap(),
            cancel_alpha: args[4].parse().unwrap(),
            invest_limit: args[5].parse().unwrap(),
            cancel_limit: args[6].parse().unwrap(),
            invest_cost: args[7].parse().unwrap(),
        }
    } else {
        Param {
            overflow_alpha_refill: 0.5,
            overflow_alpha: 2.,
            cancel_alpha_refill: 1.1,
            cancel_alpha: 5.,
            // TODO: モンテカルロで最適なターンを動的に求めた方が良い
            invest_limit: 900,
            cancel_limit: 960,
            invest_cost: 500,
        }
    }
}

fn main() {
    time::start_clock();
    let mut interactor = IOInteractor::new();
    let (input, state) = interactor.read_input();
    let param = load_params();
    let mut solver = Solver { state, param };

    solver.solve(&input, &mut interactor);
    eprintln!(
        "result: {{\"score\": {}, \"duration\": {:.4}, \"invest_level\": {}}}",
        solver.state.score,
        time::elapsed_seconds(),
        solver.state.invest_level,
    );
}
