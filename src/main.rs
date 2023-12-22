mod def;
mod interactor;
mod util;

use crate::def::*;
use crate::interactor::*;
use crate::util::*;

fn read_status(
    state: &mut State,
    input: &Input,
    interactor: &mut Interactor,
) -> Vec<(usize, i64, i64)> {
    let (projects, score, new_cards) = interactor.read_status(input);
    state.projects = projects;
    state.score = score;
    new_cards
}

fn use_card(use_card: usize, m: usize, state: &mut State, interactor: &mut Interactor) {
    interactor.output_c(use_card, m);
    state.cards[use_card] = (CardType::NONE, 0);
}

fn refill_card(
    used_card: usize,
    selected_card: usize,
    new_cards: &Vec<(usize, i64, i64)>,
    state: &mut State,
    interactor: &mut Interactor,
) {
    interactor.output_r(selected_card);
    state.cards[used_card] = (
        CardType::from_usize(new_cards[selected_card].0),
        new_cards[selected_card].1,
    );
}

fn solve(state: &mut State, input: &Input, interactor: &mut Interactor) {
    for _ in 0..input.t {
        // 1. 使うカード、使う対象の選択
        let used_card = 0;
        let m = 0;

        use_card(used_card, m, state, interactor);
        let new_cards = read_status(state, input, interactor);

        // 2. 補充するカードの選択
        let selected_card = 0;
        refill_card(used_card, selected_card, &new_cards, state, interactor);
    }
}

fn main() {
    time::start_clock();
    let mut interactor = Interactor::new();
    let (input, mut state) = interactor.read_input();

    solve(&mut state, &input, &mut interactor);
    eprintln!(
        "result: {{\"score\": {}, \"duration\": {:.4}}}",
        state.score,
        time::elapsed_seconds()
    );
}
