use itertools::Itertools;
use std::{collections::HashSet, fmt::Display, fs::File};

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

#[derive(Clone, Debug)]
enum TileBackground {
    None,
    Wall(Vec2),
    Floor(Vec2),
}

#[derive(Clone, Debug)]
struct Tile {
    background: TileBackground,
}

fn tiles_file(tiles: &[Vec<Tile>], level: usize) -> String {
    let columns = tiles
        .iter()
        .map(|column| {
            let column = column
                .iter()
                .map(|tile| {
                    let background = match tile.background {
                        TileBackground::None => "None".to_string(),
                        TileBackground::Wall(Vec2 { x, y }) => format!("Wall(vec2({x}, {y}))"),
                        TileBackground::Floor(Vec2 { x, y }) => format!("Floor(vec2({x}, {y}))"),
                    };

                    format!(
                        "Tile {{
                    background: TileBackground::{background},
                    item: Item::None,
                    player: false,
                    blood_level: BloodLevel::None
                }}"
                    )
                })
                .join(",");

            format!("vec![{column}]")
        })
        .join(",");

    format!(
        "
use crate::{{Tile, TileBackground, Item, vec2, BloodLevel}};
        
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
            let tag = ldtk.defs.tilesets[0]
                .enum_tags
                .iter()
                .find(|e| e.tile_ids.contains(&tile.tile));

            let Some(tag) = tag else {
                return;
            };

            let (x, y) = (tile.position.0 / 16, tile.position.1 / 16);

            if tag.enum_value_id == "Wall" || tag.enum_value_id == "Floor" {
                if grid.len() < x + 1 {
                    grid.extend(std::iter::repeat_n(Vec::new(), x + 1 - grid.len()));
                }

                let column = &mut grid[x];

                if column.len() < y + 1 {
                    column.extend(std::iter::repeat_n(
                        Tile {
                            background: TileBackground::None,
                        },
                        y + 1 - column.len(),
                    ));
                }

                column[y] = Tile {
                    background: if tag.enum_value_id == "Wall" {
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
                };
            }
        });

    let file = tiles_file(&grid, 0);

    std::fs::write("../src/tiles.rs", file).unwrap();
}
