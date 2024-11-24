#![allow(clippy::missing_safety_doc)]

mod tiles;

use std::collections::HashSet;
use turbo::borsh::{self, *};
use turbo::prelude::*;

const CELL_SIZE: usize = 16;
const FRAMES_BETWEEN_MOVES: usize = 16;
const BODY_CHOPPING_TIME: isize = 100;
const PROGRESS_BAR_SIZE: usize = CELL_SIZE - 4;

struct CharacterSpriteLocations {
    down: Vec2,
    right: Vec2,
    up: Vec2,
}

const THE_CAT: CharacterSpriteLocations = CharacterSpriteLocations {
    down: vec2(4, 0),
    right: vec2(4, 5),
    up: vec2(5, 5),
};
// const TWO_TOES: CharacterSpriteLocations = CharacterSpriteLocations {
//     down: vec2(5, 0),
//     right: vec2(6, 5),
//     up: vec2(7, 5),
// };
// const MACHINE_GUN: CharacterSpriteLocations = CharacterSpriteLocations {
//     down: vec2(6, 0),
//     right: vec2(8, 5),
//     up: vec2(9, 5),
// };
// const ONE_EYE: CharacterSpriteLocations = CharacterSpriteLocations {
//     down: vec2(7, 0),
//     right: vec2(10, 5),
//     up: vec2(11, 5),
// };

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy, Default)]
struct Vec2 {
    x: usize,
    y: usize,
}

struct IVec2 {
    x: isize,
    y: isize,
}

const fn vec2(x: usize, y: usize) -> Vec2 {
    Vec2 { x, y }
}

const fn ivec2(x: isize, y: isize) -> IVec2 {
    IVec2 { x, y }
}

impl std::ops::AddAssign<IVec2> for Vec2 {
    fn add_assign(&mut self, rhs: IVec2) {
        self.x = ((self.x as isize) + rhs.x) as usize;
        self.y = ((self.y as isize) + rhs.y) as usize;
    }
}

impl std::ops::Add<IVec2> for Vec2 {
    type Output = Self;

    fn add(mut self, rhs: IVec2) -> Self {
        self += rhs;

        self
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
enum TileBackground {
    Floor(Vec2),
    Wall(Vec2),
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
enum Furniture {
    None,
    Floor(Vec2),
    Wall(Vec2),
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
enum BodyLevel {
    // CHOP THIS
    Start,
    One,
    Two,
    Three,

    // BAG THIS
    Four,
    Five,
    Six,
    Seven,
}

#[derive(PartialEq)]
enum BodyAction {
    Chop,
    Bag,
}

impl BodyLevel {
    fn lower(self) -> Option<Self> {
        match self {
            BodyLevel::Start => Some(BodyLevel::One),
            BodyLevel::One => Some(BodyLevel::Two),
            BodyLevel::Two => Some(BodyLevel::Three),
            BodyLevel::Three => Some(BodyLevel::Four),
            BodyLevel::Four => Some(BodyLevel::Five),
            BodyLevel::Five => Some(BodyLevel::Six),
            BodyLevel::Six => Some(BodyLevel::Seven),
            BodyLevel::Seven => None,
        }
    }

    fn action(&self) -> BodyAction {
        match self {
            BodyLevel::Start | BodyLevel::One | BodyLevel::Two | BodyLevel::Three => {
                BodyAction::Chop
            }
            BodyLevel::Four | BodyLevel::Five | BodyLevel::Six | BodyLevel::Seven => {
                BodyAction::Bag
            }
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
enum Item {
    None,
    Body(BodyLevel, isize),
    Knife,
    Sponge,
    Bleach,
    Bag,
    BagRoll,
    BodyBag,
}

impl Item {
    fn collidable(&self) -> bool {
        match self {
            Item::None
            | Item::Body(_, _)
            | Item::Knife
            | Item::Sponge
            | Item::Bleach
            | Item::Bag
            | Item::BagRoll => false,
            Item::BodyBag => true,
        }
    }
    fn draw(&self, location: Vec2, flip: bool) {
        match self {
            Item::None => {}
            Item::Body(level, prgrss) => {
                match level {
                    BodyLevel::Start => {
                        asset(vec2(8, 0), location).draw();
                    }
                    BodyLevel::One => asset(vec2(13, 0), location).draw(),
                    BodyLevel::Two => {
                        asset(vec2(13, 0), location).draw();
                        asset(vec2(14, 0), location).draw();
                    }
                    BodyLevel::Three => {
                        asset(vec2(13, 0), location).draw();
                        asset(vec2(14, 0), location).draw();
                        asset(vec2(15, 0), location).draw();
                    }
                    BodyLevel::Four => {
                        asset(vec2(13, 0), location).draw();
                        asset(vec2(14, 0), location).draw();
                        asset(vec2(15, 0), location).draw();
                        asset(vec2(16, 0), location).draw();
                    }
                    BodyLevel::Five => {
                        asset(vec2(14, 0), location).draw();
                        asset(vec2(15, 0), location).draw();
                        asset(vec2(16, 0), location).draw();
                    }
                    BodyLevel::Six => {
                        asset(vec2(15, 0), location).draw();
                        asset(vec2(16, 0), location).draw();
                    }
                    BodyLevel::Seven => {
                        asset(vec2(16, 0), location).draw();
                    }
                }
                progress(location, (*prgrss as f32) / (BODY_CHOPPING_TIME as f32));
            }
            Item::Knife => asset(vec2(6, 2), location).flip_x(flip).draw(),
            Item::Sponge => asset(vec2(7, 2), location).flip_x(flip).draw(),
            Item::Bleach => asset(vec2(4, 2), location).flip_x(flip).draw(),
            Item::Bag => asset(vec2(9, 3), location).flip_x(flip).draw(),
            Item::BagRoll => asset(vec2(9, 2), location).flip_x(flip).draw(),
            Item::BodyBag => {
                asset(vec2(8, 2), location + ivec2(0, -1))
                    .flip_x(flip)
                    .draw();
                asset(vec2(8, 3), location).flip_x(flip).draw()
            }
        }
    }
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
    furniture: Furniture,
    player: bool,
    blood_level: BloodLevel,
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
        Grid(tiles::create_level_0())
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
    facing: Direction,
    inventory: Item,

    // Restrictions:
    disable_move_until: usize,
    last_frame_directions: HashSet<Direction>,

    // Performance:
    character_position: Vec2,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, PartialOrd, Eq, Hash, Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl From<Direction> for IVec2 {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Up => ivec2(0, -1),
            Direction::Down => ivec2(0, 1),
            Direction::Left => ivec2(-1, 0),
            Direction::Right => ivec2(1, 0),
        }
    }
}

impl GameState {
    fn move_player(&mut self, direction: Direction) {
        let previous_position = self.character_position;
        self.facing = direction;
        let new_position = self.character_position + IVec2::from(direction);
        if let TileBackground::Wall(_) = self.grid[new_position].background {
            return;
        }
        if let Furniture::Wall(_) = self.grid[new_position].furniture {
            return;
        }
        if self.grid[new_position].item.collidable() {
            return;
        }
        self.character_position = new_position;

        if let Item::Body(_, _) = self.grid[new_position].item {
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

    fn in_front_of_player(&self) -> Vec2 {
        self.character_position + IVec2::from(self.facing)
    }

    fn interact(&mut self) {
        let in_front_of_player = self.in_front_of_player();

        match &mut self.grid[in_front_of_player].item {
            Item::None => {}
            Item::Body(level, progress) => {
                if level.action() == BodyAction::Bag {
                    if self.inventory != Item::Bag {
                        return;
                    }

                    *progress -= 1;

                    if *progress <= 0 {
                        self.inventory = Item::None;
                        let Some(new_level) = level.lower() else {
                            self.grid[in_front_of_player].item = Item::None;
                            return;
                        };
                        self.grid[in_front_of_player].item =
                            Item::Body(new_level, BODY_CHOPPING_TIME);
                        let below_body = in_front_of_player + ivec2(0, 1);
                        self.grid[below_body].item = Item::BodyBag;
                    }

                    return;
                }

                if self.inventory != Item::Knife {
                    return;
                }

                *progress -= 1;

                if *progress <= 0 {
                    let Some(new_level) = level.lower() else {
                        self.grid[in_front_of_player].item = Item::None;
                        return;
                    };
                    self.grid[in_front_of_player].item = Item::Body(new_level, BODY_CHOPPING_TIME);
                }
            }
            item @ (Item::Knife | Item::Sponge | Item::Bleach | Item::Bag | Item::BodyBag) => {
                if self.inventory != Item::None {
                    return;
                }

                self.inventory = *item;
                self.grid[in_front_of_player].item = Item::None;
            }
            Item::BagRoll => {
                if self.inventory != Item::None {
                    return;
                }

                self.inventory = Item::Bag;
            }
        }
    }

    fn drop(&mut self) {
        let in_front_of_player = self.in_front_of_player();

        if self.grid[in_front_of_player].item != Item::None {
            return;
        }

        self.grid[in_front_of_player].item = self.inventory;
        self.inventory = Item::None;
    }
}

impl Default for GameState {
    fn default() -> Self {
        let mut grid = Grid::new();

        let character_position = vec2(5, 5);
        grid[character_position].player = true;
        // grid[vec2(3, 3)].item = Item::Body(BodyLevel::Start, BODY_CHOPPING_TIME);
        // grid[vec2(2, 1)].item = Item::Knife;

        GameState {
            grid,
            inventory: Item::None,
            blood_on_boots: BloodLevel::None,
            facing: Direction::Down,
            character_position,
            disable_move_until: 0,
            last_frame_directions: HashSet::new(),
        }
    }
}

#[must_use]
struct Asset {
    sprite: Vec2,
    location: Vec2,
    opacity: f32,
    flip_x: bool,
}

fn asset(sprite: Vec2, location: Vec2) -> Asset {
    Asset::new(sprite, location)
}

impl Asset {
    fn new(sprite: Vec2, location: Vec2) -> Asset {
        Asset {
            sprite,
            location,
            opacity: 1.0,
            flip_x: false,
        }
    }

    #[allow(clippy::neg_multiply)]
    fn draw(self) {
        sprite!(
            "assets",
            x = CELL_SIZE * self.location.x,
            y = CELL_SIZE * self.location.y,
            w = CELL_SIZE,
            h = CELL_SIZE,
            sw = CELL_SIZE,
            sh = CELL_SIZE,
            sx = self.sprite.x * 16,
            sy = self.sprite.y * 16,
            opacity = self.opacity,
            flip_x = self.flip_x
        );
    }

    fn flip_x(self, flip_x: bool) -> Asset {
        Asset { flip_x, ..self }
    }
}

fn progress(location: Vec2, progress: f32) {
    let rect_size = lerp(PROGRESS_BAR_SIZE, 0, progress);
    rect!(
        x = CELL_SIZE * location.x + (CELL_SIZE - PROGRESS_BAR_SIZE) / 2,
        y = CELL_SIZE * location.y - CELL_SIZE / 4,
        w = PROGRESS_BAR_SIZE,
        h = 2,
        color = 0x888888ff,
    );
    rect!(
        x = CELL_SIZE * location.x + (CELL_SIZE - PROGRESS_BAR_SIZE) / 2,
        y = CELL_SIZE * location.y - CELL_SIZE / 4,
        w = rect_size,
        h = 2,
        color = 0x00ff00ff,
    );
}

fn lerp(start: usize, end: usize, t: f32) -> usize {
    ((start as f32) * (1.0 - t) + (end as f32) * t).round() as usize
}

fn update(mut state: GameState) -> GameState {
    state.grid.iter().enumerate().for_each(|(row_index, row)| {
        row.iter().enumerate().for_each(|(column_index, cell)| {
            let location = vec2(row_index, column_index);
            asset(
                match cell.background {
                    TileBackground::Floor(sprite) => sprite,
                    TileBackground::Wall(sprite) => sprite,
                },
                location,
            )
            .draw();

            if cell.blood_level != BloodLevel::None {
                match cell.blood_level {
                    BloodLevel::None => unreachable!(),
                    BloodLevel::Tall => asset(vec2(7, 1), location).draw(),
                    BloodLevel::Grande => {
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
                    BloodLevel::Venti => asset(vec2(5, 1), location).draw(),
                };
            }

            if let Furniture::Floor(sprite) | Furniture::Wall(sprite) = cell.furniture {
                asset(sprite, location).draw();
            }

            cell.item.draw(location, false);

            if cell.player {
                let character_sprite_locations = THE_CAT;
                let (sprite, flip) = match state.facing {
                    Direction::Up => (character_sprite_locations.up, false),
                    Direction::Down => (character_sprite_locations.down, false),
                    Direction::Left => (character_sprite_locations.right, true),
                    Direction::Right => (character_sprite_locations.right, false),
                };
                asset(sprite, location).flip_x(flip).draw();
            }

            let in_front_of_player = state.in_front_of_player();

            state
                .inventory
                .draw(in_front_of_player, matches!(state.facing, Direction::Left));
        })
    });

    let pad = gamepad(0);

    if state.disable_move_until <= tick() {
        let mut pressed = HashSet::new();
        if pad.up.pressed() {
            pressed.insert(Direction::Up);
        }
        if pad.down.pressed() {
            pressed.insert(Direction::Down);
        }
        if pad.left.pressed() {
            pressed.insert(Direction::Left);
        }
        if pad.right.pressed() {
            pressed.insert(Direction::Right);
        }

        let direction = if pressed.len() >= 2 {
            let mut difference = pressed.difference(&state.last_frame_directions);

            let direction = difference.next();

            Some(direction.copied().unwrap_or(state.facing))
        } else if pressed.len() == 1 {
            Some(pressed.into_iter().next().expect("unreachable"))
        } else {
            None
        };

        if let Some(direction) = direction {
            state.move_player(direction);
            state.disable_move_until = tick() + FRAMES_BETWEEN_MOVES;
        }
    }

    if pad.a.pressed() {
        state.interact();
    }

    if pad.b.pressed() {
        state.drop();
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
