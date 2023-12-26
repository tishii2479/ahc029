mod def;
mod interactor;
mod simulator;
mod solver;
mod util;

use crate::def::*;
use crate::interactor::*;
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
            overflow_alpha_all_refill: args[8].parse().unwrap(),
            overflow_alpha_all: args[9].parse().unwrap(),
            cancel_alpha_all_refill: args[10].parse().unwrap(),
            cancel_alpha_all: args[11].parse().unwrap(),
            work_single_beta: args[12].parse().unwrap(),
        }
    } else {
        Param {
            cancel_alpha: 3.6968886682561326,
            cancel_alpha_refill: 1.0809478571856825,
            cancel_alpha_all: 3.6968886682561326,
            cancel_alpha_all_refill: 1.0809478571856825,
            cancel_limit: 979,
            invest_cost: 510,
            invest_limit: 863,
            overflow_alpha: 2.016646721749814,
            overflow_alpha_all: 2.7694849713061416,
            overflow_alpha_all_refill: 0.5531191229327318,
            overflow_alpha_refill: 0.25378716369770815,
            work_single_beta: 2.,
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
