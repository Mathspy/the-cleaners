#![allow(clippy::missing_safety_doc)]

// This is where your main game loop code goes
// The stuff in this block will run ~60x per sec

use turbo::borsh::{self, *};
use turbo::prelude::*;

const FRAMES_BETWEEN_MOVES: usize = 16;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
enum Tile {
    Black,
    White,
}

impl Tile {
    fn flip(self) -> Self {
        match self {
            Tile::Black => Tile::White,
            Tile::White => Tile::Black,
        }
    }
}

impl From<&Tile> for u32 {
    fn from(value: &Tile) -> Self {
        match value {
            Tile::Black => 0x111111ff,
            Tile::White => 0xffffffff,
        }
    }
}

enum Parity {
    Even,
    Odd,
}

impl From<u32> for Parity {
    fn from(value: u32) -> Parity {
        if value % 2 == 0 {
            Parity::Even
        } else {
            Parity::Odd
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct GameState {
    grid: Vec<Vec<Tile>>,
    character_position: (u32, u32),
    disable_move_until: usize,
}

impl Default for GameState {
    fn default() -> Self {
        let grid = (0..20)
            .map(|x| {
                let start_with = if matches!(Parity::from(x), Parity::Even) {
                    Tile::Black
                } else {
                    Tile::White
                };

                (0..20)
                    .map(|y| {
                        let parity = Parity::from(y);

                        match parity {
                            Parity::Even => start_with.flip(),
                            Parity::Odd => start_with,
                        }
                    })
                    .collect()
            })
            .collect();

        GameState {
            grid,
            character_position: (0, 0),
            disable_move_until: 0,
        }
    }
}

fn update(mut state: GameState) -> GameState {
    state
        .grid
        .iter()
        .enumerate()
        .for_each(|(column_index, row)| {
            row.iter().enumerate().for_each(|(row_index, cell)| {
                rect!(
                    w = 16,
                    h = 16,
                    x = 16 * row_index,
                    y = 16 * column_index,
                    color = u32::from(cell)
                );
            })
        });

    rect!(
        w = 16,
        h = 16,
        x = 16 * state.character_position.0,
        y = 16 * state.character_position.1,
        color = 0xff4f00ff
    );

    let pad = gamepad(0);

    if state.disable_move_until <= tick() {
        let mut moved = false;
        if pad.up.pressed() {
            state.character_position.1 -= 1;
            moved = true;
        }
        if pad.down.pressed() {
            state.character_position.1 += 1;
            moved = true;
        }
        if pad.left.pressed() {
            state.character_position.0 -= 1;
            moved = true;
        }
        if pad.right.pressed() {
            state.character_position.0 += 1;
            moved = true;
        }

        if moved {
            state.disable_move_until = tick() + FRAMES_BETWEEN_MOVES;
            set_cam!(
                x = state.character_position.0 * 16,
                y = state.character_position.1 * 16
            );
        }
    }

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

// #[cfg(not(no_run))]
#[no_mangle]
#[allow(overflowing_literals, non_upper_case_globals)]
// # Safety: builtin turbo function run
pub unsafe extern "C" fn run() {
    inner_update()
}

// #[cfg(no_run)]
// #[allow(overflowing_literals, non_upper_case_globals)]
// unsafe fn run() {
//     inner_update()
// }

// #[cfg(not(no_run))]
pub fn run_snapshot(snapshot_data: &[u8]) -> Vec<u8> {
    turbo::run_snapshot(snapshot_data, || unsafe { run() })
}
