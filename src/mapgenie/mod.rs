//! MapGenie: pages and one JSON endpoint of mapgenie.io, turned into the SDK's wire.

mod convert;
mod data;
mod html;
mod page;
#[cfg(test)]
mod tests;

use sdk::guest::{Context, http_get};
use sdk::wire::{Error, Map, MapOffer, Title};

use self::data::{Data, IconManifest};
use crate::kit::text::is_slug;
use crate::kit::{get_text, internal, not_found};

pub fn titles(_: &Context) -> Result<Vec<Title>, Error> {
    Ok(html::parse_home(&get_text("https://mapgenie.io/")?))
}

pub fn maps(_: &Context, title: &str) -> Result<Vec<MapOffer>, Error> {
    if !is_slug(title) {
        return Err(not_found(title));
    }
    Ok(html::parse_game(title, &get_text(&format!("https://mapgenie.io/{title}"))?))
}

pub fn map(_: &Context, map: &str) -> Result<Map, Error> {
    let (game, name) = map.split_once('/').filter(|(game, name)| is_slug(game) && is_slug(name)).ok_or_else(|| not_found(map))?;
    let page = page::parse_map_page(&get_text(&format!("https://mapgenie.io/{game}/maps/{name}"))?)?;
    let manifest = match &page.icon_manifest_url {
        Some(url) => serde_json::from_slice(&http_get(url)?).map_err(|e| internal(format!("icon manifest: {e}")))?,
        None => IconManifest::default(),
    };
    let body = http_get(&format!("https://mapgenie.io/api/v1/maps/{}/data", page.map.id))?;
    let data: Data = serde_json::from_slice(&body).map_err(|e| internal(format!("map data: {e}")))?;
    convert::convert(page, &manifest, data)
}
