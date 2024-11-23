#![allow(clippy::missing_safety_doc)]

// This is where your main game loop code goes
// The stuff in this block will run ~60x per sec

use turbo::borsh::{self, *};
use turbo::prelude::*;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct GameState {}

impl Default for GameState {
    fn default() -> Self {
        GameState {}
    }
}

fn update(state: GameState) -> GameState {
    text!("Hello world!");

    state
}

// UPDATE MACHINERY
fn inner_update() {
    let state = turbo::sys::load()
        .and_then(|xs| GameState::try_from_slice(xs).map_err(|_| -1))
        .unwrap_or_default();
    std::println!("Loaded {:?}", state);

    let state = update(state);

    if let Ok(bytes) = GameState::try_to_vec(&state) {
        if turbo::sys::save(&bytes).is_ok() {
            std::println!("Saved {:?}", state);
        }
    }
}

#[cfg(not(no_run))]
#[no_mangle]
#[allow(overflowing_literals, non_upper_case_globals)]
// # Safety: builtin turbo function run
pub unsafe extern "C" fn run() {
    inner_update()
}

#[cfg(no_run)]
#[allow(overflowing_literals, non_upper_case_globals)]
unsafe fn run() {
    inner_update()
}

#[cfg(not(no_run))]
pub fn run_snapshot(snapshot_data: &[u8]) -> Vec<u8> {
    turbo::run_snapshot(snapshot_data, || unsafe { run() })
}
