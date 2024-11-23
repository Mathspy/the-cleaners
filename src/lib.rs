#![allow(clippy::missing_safety_doc)]

// This is where your main game loop code goes
// The stuff in this block will run ~60x per sec

use turbo::borsh::{self, *};
use turbo::prelude::*;

const CELL_SIZE: usize = 16;
const FRAMES_BETWEEN_MOVES: usize = 16;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy, Default)]
struct Vec2 {
    x: usize,
    y: usize,
}

fn vec2(x: usize, y: usize) -> Vec2 {
    Vec2 { x, y }
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
enum Item {
    None,
    Body,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
enum BloodLevel {
    None,
    Tall,
    Grande,
    Venti,
}

impl BloodLevel {
    /// Returns whether incrementing succeeded or not
    fn increment(&mut self) -> bool {
        match self {
            BloodLevel::None => {
                *self = BloodLevel::Tall;
                true
            }
            BloodLevel::Tall => {
                *self = BloodLevel::Grande;
                true
            }
            BloodLevel::Grande => {
                *self = BloodLevel::Venti;
                true
            }
            BloodLevel::Venti => false,
        }
    }

    /// Returns whether decrementing succeeded or not
    fn decrement(&mut self) -> bool {
        match self {
            BloodLevel::None => false,
            BloodLevel::Tall => {
                *self = BloodLevel::None;
                true
            }
            BloodLevel::Grande => {
                *self = BloodLevel::Tall;
                true
            }
            BloodLevel::Venti => {
                *self = BloodLevel::Grande;
                true
            }
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
struct Tile {
    background: TileBackground,
    item: Item,
    player: bool,
    blood_level: BloodLevel,
}

impl Tile {
    fn from_background(background: TileBackground) -> Tile {
        Tile {
            background,
            item: Item::None,
            player: false,
            blood_level: BloodLevel::None,
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
struct Grid(Vec<Vec<Tile>>);

impl Grid {
    fn new() -> Self {
        Grid(
            (0..20)
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
                                Parity::Even => Tile::from_background(start_with.flip()),
                                Parity::Odd => Tile::from_background(start_with),
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
        )
    }

    fn iter(&self) -> std::slice::Iter<'_, Vec<Tile>> {
        self.0.iter()
    }
}

impl std::ops::Index<Vec2> for Grid {
    type Output = Tile;

    fn index(&self, index: Vec2) -> &Self::Output {
        &self.0[index.x][index.y]
    }
}

impl std::ops::IndexMut<Vec2> for Grid {
    fn index_mut(&mut self, index: Vec2) -> &mut Self::Output {
        &mut self.0[index.x][index.y]
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
struct GameState {
    // Gameplay:
    grid: Grid,
    blood_on_boots: BloodLevel,

    // Restrictions:
    disable_move_until: usize,

    // Performance:
    character_position: Vec2,
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

        if self.grid[new_position].item == Item::Body {
            self.grid[new_position].blood_level = BloodLevel::Venti;
            self.blood_on_boots = BloodLevel::Venti;
        }

        if self.blood_on_boots != BloodLevel::None {
            let some_blood_drained = self.grid[new_position].blood_level.increment();
            if some_blood_drained {
                self.blood_on_boots.decrement();
            }
        }

        self.grid[previous_position].player = false;
        self.grid[new_position].player = true
    }
}

impl Default for GameState {
    fn default() -> Self {
        let mut grid = Grid::new();

        grid[vec2(0, 0)].player = true;
        grid[vec2(3, 3)].item = Item::Body;

        GameState {
            grid,
            blood_on_boots: BloodLevel::None,
            character_position: Vec2::new(),
            disable_move_until: 0,
        }
    }
}

fn update(mut state: GameState) -> GameState {
    state.grid.iter().enumerate().for_each(|(row_index, row)| {
        row.iter().enumerate().for_each(|(column_index, cell)| {
            rect!(
                w = CELL_SIZE,
                h = CELL_SIZE,
                x = CELL_SIZE * row_index,
                y = CELL_SIZE * column_index,
                color = u32::from(&cell.background)
            );

            match cell.item {
                Item::None => {}
                Item::Body => {
                    rect!(
                        w = CELL_SIZE,
                        h = CELL_SIZE,
                        x = CELL_SIZE * row_index,
                        y = CELL_SIZE * column_index,
                        color = 0x0000ffff
                    );
                }
            }

            if cell.blood_level != BloodLevel::None {
                let diameter = match cell.blood_level {
                    BloodLevel::None => unreachable!(),
                    BloodLevel::Tall => 4,
                    BloodLevel::Grande => 8,
                    BloodLevel::Venti => 14,
                };
                circ!(
                    x = CELL_SIZE * row_index + CELL_SIZE / 2 - diameter / 2,
                    y = CELL_SIZE * column_index + CELL_SIZE / 2 - diameter / 2,
                    d = diameter,
                    color = 0xff000077
                );
            }

            if cell.player {
                rect!(
                    w = CELL_SIZE,
                    h = CELL_SIZE,
                    x = CELL_SIZE * row_index,
                    y = CELL_SIZE * column_index,
                    color = 0xff4f00ff
                );
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
