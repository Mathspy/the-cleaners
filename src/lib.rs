#![allow(clippy::missing_safety_doc)]

// This is where your main game loop code goes
// The stuff in this block will run ~60x per sec

use turbo::borsh::{self, *};
use turbo::prelude::*;

const FRAMES_BETWEEN_MOVES: usize = 16;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy, Default)]
struct Vec2 {
    x: usize,
    y: usize,
}

impl Vec2 {
    fn new() -> Self {
        Default::default()
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
enum TileBackground {
    Black,
    White,
}

impl TileBackground {
    fn flip(self) -> Self {
        match self {
            TileBackground::Black => TileBackground::White,
            TileBackground::White => TileBackground::Black,
        }
    }
}

impl From<&TileBackground> for u32 {
    fn from(value: &TileBackground) -> Self {
        match value {
            TileBackground::Black => 0x111111ff,
            TileBackground::White => 0xffffffff,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
enum TileForeground {
    Empty,
    Player,
    Body,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
struct Tile {
    background: TileBackground,
    foreground: TileForeground,
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
    character_position: Vec2,
    disable_move_until: usize,
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl GameState {
    fn move_player(&mut self, direction: Direction) {
        let previous_position = self.character_position;
        match direction {
            Direction::Up => self.character_position.y -= 1,
            Direction::Down => self.character_position.y += 1,
            Direction::Left => self.character_position.x -= 1,
            Direction::Right => self.character_position.x += 1,
        };
        let new_position = self.character_position;

        self.grid[previous_position.x][previous_position.y].foreground = TileForeground::Empty;
        self.grid[new_position.x][new_position.y].foreground = TileForeground::Player
    }
}

impl Default for GameState {
    fn default() -> Self {
        let mut grid = (0..20)
            .map(|x| {
                let start_with = if matches!(Parity::from(x), Parity::Even) {
                    TileBackground::Black
                } else {
                    TileBackground::White
                };

                (0..20)
                    .map(|y| {
                        let parity = Parity::from(y);

                        match parity {
                            Parity::Even => Tile {
                                background: start_with.flip(),
                                foreground: TileForeground::Empty,
                            },
                            Parity::Odd => Tile {
                                background: start_with,
                                foreground: TileForeground::Empty,
                            },
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        grid[0][0].foreground = TileForeground::Player;

        GameState {
            grid,
            character_position: Vec2::new(),
            disable_move_until: 0,
        }
    }
}

fn update(mut state: GameState) -> GameState {
    state.grid.iter().enumerate().for_each(|(row_index, row)| {
        row.iter().enumerate().for_each(|(column_index, cell)| {
            rect!(
                w = 16,
                h = 16,
                x = 16 * row_index,
                y = 16 * column_index,
                color = u32::from(&cell.background)
            );

            match cell.foreground {
                TileForeground::Empty => {}
                TileForeground::Player => {
                    rect!(
                        w = 16,
                        h = 16,
                        x = 16 * row_index,
                        y = 16 * column_index,
                        color = 0xff4f00ff
                    );
                }
                TileForeground::Body => todo!(),
            }
        })
    });

    let pad = gamepad(0);

    if state.disable_move_until <= tick() {
        let mut moved = false;
        if pad.up.pressed() {
            log!("UP IS CLICKED");
            state.move_player(Direction::Up);
            moved = true;
        }
        if pad.down.pressed() {
            state.move_player(Direction::Down);
            moved = true;
        }
        if pad.left.pressed() {
            state.move_player(Direction::Left);
            moved = true;
        }
        if pad.right.pressed() {
            state.move_player(Direction::Right);
            moved = true;
        }

        if moved {
            state.disable_move_until = tick() + FRAMES_BETWEEN_MOVES;
        }
    }

    set_cam!(
        x = state.character_position.x * 16,
        y = state.character_position.y * 16
    );

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
