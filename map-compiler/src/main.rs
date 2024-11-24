use itertools::Itertools;
use std::{collections::HashSet, fs::File};

use serde::Deserialize;
use serde_json::{self};

#[derive(Deserialize)]
struct EnumTag {
    #[serde(rename = "enumValueId")]
    enum_value_id: String,
    #[serde(rename = "tileIds")]
    tile_ids: HashSet<usize>,
}

#[derive(Deserialize, Debug)]
struct CustomData {
    #[serde(rename = "tileId")]
    tile_id: usize,
    data: String,
}

#[derive(Deserialize)]
struct Tileset {
    #[serde(rename = "enumTags")]
    enum_tags: Vec<EnumTag>,
    #[serde(rename = "customData")]
    custom_data: Vec<CustomData>,
}

#[derive(Deserialize)]
struct Defs {
    tilesets: Vec<Tileset>,
}

#[derive(Deserialize, Debug)]
struct GridTile {
    #[serde(rename = "px")]
    position: (usize, usize),
    #[serde(rename = "src")]
    source: (usize, usize),
    #[serde(rename = "t")]
    tile: usize,
}

#[derive(Deserialize)]
struct LayerInstance {
    #[serde(rename = "__identifier")]
    id: String,
    #[serde(rename = "gridTiles")]
    grid_tiles: Vec<GridTile>,
}

#[derive(Deserialize)]
struct Level {
    #[serde(rename = "layerInstances")]
    layer_instances: Vec<LayerInstance>,
}

#[derive(Deserialize)]
struct Ldtk {
    defs: Defs,
    levels: Vec<Level>,
}

#[derive(Clone, Debug)]
struct Vec2 {
    x: usize,
    y: usize,
}

#[derive(Clone, Debug, Default)]
enum TileBackground {
    #[default]
    None,
    Wall(Vec2),
    Floor(Vec2),
}

#[derive(Clone, Debug, Default)]
enum Item {
    #[default]
    None,
    Bleach,
    Knife,
    Sponge,
    BagRoll,
    BodyBag,
    Bag,
    Body,
}

#[derive(Clone, Debug, Default)]
enum BloodLevel {
    #[default]
    None,
    Tall,
    Grande,
    Venti,
}

#[derive(Clone, Debug, Default)]
struct Tile {
    background: TileBackground,
    furniture: TileBackground,
    item: Item,
    blood_level: BloodLevel,
    drop_point: bool,
}

fn tiles_file(tiles: &[Vec<Tile>], level: usize) -> String {
    let columns = tiles
        .iter()
        .enumerate()
        .map(|(x, column)| {
            let column = column
                .iter()
                .enumerate()
                .map(|(y, tile)| {
                    let background = match tile.background {
                        TileBackground::None => panic!("Invalid Tilebackground at {x}, {y}"),
                        TileBackground::Wall(Vec2 { x, y }) => format!("Wall(vec2({x}, {y}))"),
                        TileBackground::Floor(Vec2 { x, y }) => format!("Floor(vec2({x}, {y}))"),
                    };

                    let furniture = match tile.furniture {
                        TileBackground::None => "None".to_string(),
                        TileBackground::Wall(Vec2 { x, y }) => format!("Wall(vec2({x}, {y}))"),
                        TileBackground::Floor(Vec2 { x, y }) => format!("Floor(vec2({x}, {y}))"),
                    };

                    let item = match tile.item {
                        Item::None => "None".to_string(),
                        Item::Bleach => "Bleach".to_string(),
                        Item::Knife => "Knife".to_string(),
                        Item::Sponge => "Sponge".to_string(),
                        Item::BagRoll => "BagRoll".to_string(),
                        Item::BodyBag => "BodyBag".to_string(),
                        Item::Bag => "Bag".to_string(),
                        Item::Body => "Body(BodyLevel::Start, BODY_CHOPPING_TIME)".to_string(),
                    };

                    let blood_level = match tile.blood_level {
                        BloodLevel::None => "None".to_string(),
                        BloodLevel::Tall => "Tall(CLEANING_TIME)".to_string(),
                        BloodLevel::Grande => "Grande(CLEANING_TIME)".to_string(),
                        BloodLevel::Venti => "Venti(CLEANING_TIME)".to_string(),
                    };

                    let drop_point = tile.drop_point;

                    format!(
                        "Tile {{
                    background: TileBackground::{background},
                    furniture: Furniture::{furniture},
                    item: Item::{item},
                    player: false,
                    blood_level: BloodLevel::{blood_level},
                    drop_point: {drop_point},
                }}"
                    )
                })
                .join(",");

            format!("vec![{column}]")
        })
        .join(",");

    format!(
        "
use crate::{{Tile, TileBackground, Item, vec2, BloodLevel, BodyLevel, BODY_CHOPPING_TIME, Furniture, CLEANING_TIME}};
        
pub fn create_level_{level}() -> Vec<Vec<Tile>> {{
    vec![
        {columns}
    ]
}}
"
    )
}

fn main() {
    let ldtk = serde_json::from_reader::<_, Ldtk>(File::open("./Cleaners.ldtk").unwrap()).unwrap();

    let mut grid: Vec<Vec<Tile>> = Vec::new();

    ldtk.levels[0].layer_instances[2]
        .grid_tiles
        .iter()
        .for_each(|tile| {
            let tags = ldtk.defs.tilesets[0]
                .enum_tags
                .iter()
                .filter(|e| e.tile_ids.contains(&tile.tile))
                .map(|e| e.enum_value_id.clone())
                .collect::<HashSet<String>>();

            let custom_data = ldtk.defs.tilesets[0]
                .custom_data
                .iter()
                .find(|e| e.tile_id == tile.tile);

            let (x, y) = (tile.position.0 / 16, tile.position.1 / 16);

            if grid.len() < x + 1 {
                grid.extend(std::iter::repeat_n(Vec::new(), x + 1 - grid.len()));
            }

            let column = &mut grid[x];

            if column.len() < y + 1 {
                column.extend(std::iter::repeat_n(
                    Default::default(),
                    y + 1 - column.len(),
                ));
            }

            if let Some(custom_data) = custom_data {
                let data = custom_data.data.as_str();
                let prev = std::mem::take(&mut column[y]);

                column[y] = match data {
                    "BLOOD_2" => Tile {
                        blood_level: BloodLevel::Grande,
                        ..prev
                    },
                    "BLOOD_1" => Tile {
                        blood_level: BloodLevel::Tall,
                        ..prev
                    },
                    "BLOOD_3" => Tile {
                        blood_level: BloodLevel::Venti,
                        ..prev
                    },
                    "BLEACH" => Tile {
                        item: Item::Bleach,
                        ..prev
                    },
                    "KNIFE" => Tile {
                        item: Item::Knife,
                        ..prev
                    },
                    "SPONGE" => Tile {
                        item: Item::Sponge,
                        ..prev
                    },
                    "BAG_ROLL" => Tile {
                        item: Item::BagRoll,
                        ..prev
                    },
                    "BODY_BAG" => Tile {
                        item: Item::BodyBag,
                        ..prev
                    },
                    "BAG" => Tile {
                        item: Item::Bag,
                        ..prev
                    },
                    "BODY" => Tile {
                        item: Item::Body,
                        ..prev
                    },
                    "DROP_POINT" => Tile {
                        drop_point: true,
                        ..prev
                    },
                    value => panic!("Unknown item {value}"),
                }
            }

            if tags.is_empty() {
                return;
            };

            if tags.contains("Wall") || tags.contains("Floor") {
                let prev = std::mem::take(&mut column[y]);
                if tags.contains("Furniture") {
                    column[y] = Tile {
                        furniture: if tags.contains("Wall") {
                            TileBackground::Wall(Vec2 {
                                x: tile.source.0 / 16,
                                y: tile.source.1 / 16,
                            })
                        } else {
                            TileBackground::Floor(Vec2 {
                                x: tile.source.0 / 16,
                                y: tile.source.1 / 16,
                            })
                        },
                        ..prev
                    };
                } else {
                    column[y] = Tile {
                        background: if tags.contains("Wall") {
                            TileBackground::Wall(Vec2 {
                                x: tile.source.0 / 16,
                                y: tile.source.1 / 16,
                            })
                        } else {
                            TileBackground::Floor(Vec2 {
                                x: tile.source.0 / 16,
                                y: tile.source.1 / 16,
                            })
                        },
                        ..prev
                    };
                }
            }
        });

    let file = tiles_file(&grid, 0);

    std::fs::write("../src/tiles.rs", file).unwrap();
}
