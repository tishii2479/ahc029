mod def;
mod interactor;
// mod simulator;
mod solver;
mod util;

use crate::interactor::*;
// use crate::simulator::*;
use crate::solver::*;
use crate::util::*;

fn main() {
    time::start_clock();
    let mut interactor = IOInteractor::new();
    let (input, state) = interactor.read_input();
    let mut solver = Solver { state };

    solver.solve(&input, &mut interactor);
    eprintln!(
        "result: {{\"score\": {}, \"duration\": {:.4}, \"invest_level\": {}}}",
        solver.state.score,
        time::elapsed_seconds(),
        solver.state.invest_level,
    );
}
