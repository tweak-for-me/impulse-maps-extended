//! MapGenie's home page and a game page, scanned by hand.

use std::collections::{HashMap, HashSet};

use sdk::wire::{MapOffer, Title};
use serde::Deserialize;

use super::page::{MapRef, window};
use crate::kit::text::{between, is_slug, unescape};

const STORES: &str = include_str!("stores.json");
const SITE: &str = "https://mapgenie.io/";

/// Every game card of the home page; a card may link straight to the game's only map.
pub fn parse_home(html: &str) -> Vec<Title> {
    let stores: HashMap<String, Vec<String>> = serde_json::from_str(STORES).unwrap_or_default();
    let mut seen = HashSet::new();
    let mut titles = Vec::new();
    for card in html.split("game-item game-").skip(1) {
        let Some(tag) = card.split('>').next() else { continue };
        let Some(path) = between(tag, "href=\"https://mapgenie.io/", "\"") else { continue };
        let slug = path.split(['/', '?']).next().unwrap_or_default();
        let Some(name) = between(card, "card-title\">", "</h4>") else { continue };
        if !is_slug(slug) || !seen.insert(slug) {
            continue;
        }
        titles.push(Title { id: slug.to_owned(), name: unescape(name.trim()), stores: stores.get(slug).cloned().unwrap_or_default() });
    }
    titles
}

/// A map card per `/maps/` link of the game; a page without one that is a map itself offers that map.
pub fn parse_game(slug: &str, html: &str) -> Vec<MapOffer> {
    let prefix = format!("href=\"{SITE}{slug}/maps/");
    let mut seen = HashSet::new();
    let mut offers = Vec::new();
    for chunk in html.split(prefix.as_str()).skip(1) {
        let card = chunk.split("</a>").next().unwrap_or(chunk);
        let Some(map) = card.split(['"', '?']).next().filter(|map| is_slug(map)) else { continue };
        // A plain link (a menu, a breadcrumb) has no title: only a card is a map offer.
        let Some(name) = between(card, "card-title\">", "</h4>") else { continue };
        if !seen.insert(map) {
            continue;
        }
        let name = unescape(name.trim());
        let thumbnail = card
            .find("card-img-top")
            .and_then(|at| card[at..].split('>').next())
            .and_then(|img| between(img, "src=\"", "\""))
            .map(unescape);
        offers.push(MapOffer { id: format!("{slug}/{map}"), name, thumbnail, revision: None });
    }
    if offers.is_empty() {
        offers.extend(own_map(html).map(|map| MapOffer {
            id: format!("{slug}/{}", map.slug),
            name: map.title,
            thumbnail: None,
            revision: None,
        }));
    }
    offers
}

fn own_map(html: &str) -> Option<MapRef> {
    #[derive(Deserialize)]
    struct Only {
        map: MapRef,
    }
    window::<Only>(html, "mapData").map(|only| only.map)
}
