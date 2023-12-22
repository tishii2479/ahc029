mod def;
mod interactor;
mod util;

use crate::def::*;
use crate::interactor::*;
use crate::util::*;

fn solve(input: &Input, interactor: &mut Interactor) -> i64 {
    let mut score = 0;
    for _ in 0..input.t {
        interactor.output_c(0, 0);
        (_, score, _) = interactor.read_status(input);
        interactor.output_r(0);
    }
    score
}

fn main() {
    time::start_clock();
    let mut interactor = Interactor::new();
    let input = interactor.read_input();

    let score = solve(&input, &mut interactor);
    eprintln!(
        "result: {{\"score\": {}, \"duration\": {:.4}}}",
        score,
        time::elapsed_seconds()
    );
}
