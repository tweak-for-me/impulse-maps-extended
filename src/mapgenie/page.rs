//! What a map page carries in its inline scripts: `window.mapData`, `window.game`, `window.tilesCdnUrl`.

use sdk::wire::Error;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::kit::internal;

const TILES_CDN: &str = "https://tiles.mapgenie.io/games/";

pub struct Page {
    pub tiles_cdn: String,
    pub game_slug: String,
    pub map: MapRef,
    pub tile_sets: Vec<TileSetData>,
    pub groups: Vec<GroupData>,
    pub icon_manifest_url: Option<String>,
}

#[derive(Deserialize)]
pub struct MapRef {
    pub id: u64,
    pub title: String,
    pub slug: String,
}

#[derive(Deserialize)]
pub struct TileSetData {
    pub id: u64,
    pub name: String,
    pub pattern: String,
    pub min_zoom: u8,
    pub tiles_max_zoom: u8,
    #[serde(default)]
    pub order: f64,
}

#[derive(Deserialize)]
pub struct GroupData {
    pub title: String,
    #[serde(default)]
    pub order: f64,
    /// Hex RGB without a hash.
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub categories: Vec<CategoryData>,
}

#[derive(Deserialize)]
pub struct CategoryData {
    pub id: u64,
    pub title: String,
    #[serde(default)]
    pub order: f64,
}

#[derive(Deserialize)]
struct MapData {
    map: MapRef,
    #[serde(rename = "mapConfig")]
    map_config: MapConfig,
    #[serde(default)]
    groups: Vec<GroupData>,
}

#[derive(Deserialize)]
struct MapConfig {
    tile_sets: Vec<TileSetData>,
}

#[derive(Deserialize)]
struct Game {
    slug: String,
    #[serde(default)]
    config: GameConfig,
}

#[derive(Deserialize, Default)]
struct GameConfig {
    #[serde(default)]
    icon_manifest_url: Option<String>,
}

/// The JSON value assigned to `window.<name>`: it ends where the value does, whatever follows.
pub fn window<T: DeserializeOwned>(html: &str, name: &str) -> Option<T> {
    let marker = format!("window.{name} = ");
    let rest = &html[html.find(&marker)? + marker.len()..];
    serde_json::Deserializer::from_str(rest).into_iter::<T>().next()?.ok()
}

pub fn parse_map_page(html: &str) -> Result<Page, Error> {
    let data: MapData = window(html, "mapData").ok_or_else(|| internal("the page has no readable map data"))?;
    let game: Game = window(html, "game").ok_or_else(|| internal("the page has no readable game"))?;
    Ok(Page {
        tiles_cdn: window(html, "tilesCdnUrl").unwrap_or_else(|| TILES_CDN.to_owned()),
        game_slug: game.slug,
        map: data.map,
        tile_sets: data.map_config.tile_sets,
        groups: data.groups,
        icon_manifest_url: game.config.icon_manifest_url,
    })
}
