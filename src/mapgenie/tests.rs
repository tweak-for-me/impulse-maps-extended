//! Native tests on the saved pages of fixtures/mapgenie (gitignored), so ignored by default: `cargo test -- --include-ignored`.

use sdk::guest::mercator;
use sdk::wire::{Icon, Map, PIN_LINK};

use super::convert::convert;
use super::data::{Data, IconManifest};
use super::html::{parse_game, parse_home};
use super::page::parse_map_page;

macro_rules! research {
    ($file:literal) => {
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/mapgenie/", $file))
            .unwrap_or_else(|e| panic!("fixtures/mapgenie/{}: {e}", $file))
    };
}

fn night_city(page: &str, manifest: &str, data: &str) -> Map {
    let manifest: IconManifest = serde_json::from_str(manifest).unwrap();
    convert(parse_map_page(page).unwrap(), &manifest, serde_json::from_str::<Data>(data).unwrap()).unwrap()
}

#[test]
#[ignore = "needs fixtures/mapgenie"]
fn the_home_page_gives_every_game_with_its_stores() {
    let titles = parse_home(&research!("pages/home.html"));
    assert!(titles.len() >= 200, "{}", titles.len());
    let cyberpunk = titles.iter().find(|t| t.id == "cyberpunk-2077").unwrap();
    assert_eq!(cyberpunk.name, "Cyberpunk 2077");
    assert!(cyberpunk.stores.contains(&"steam:1091500".to_owned()));
    assert!(titles.iter().any(|t| t.name.contains("Assassin's Creed")), "an &#039; is unescaped");
    assert!(titles.iter().all(|t| !t.name.contains("&#039;") && !t.name.contains("&amp;")));
}

#[test]
#[ignore = "needs fixtures/mapgenie"]
fn a_game_page_offers_its_map_with_the_preview() {
    let offers = parse_game("cyberpunk-2077", &research!("pages/cyberpunk-2077.html"));
    assert_eq!(offers.len(), 1);
    assert_eq!((offers[0].id.as_str(), offers[0].name.as_str()), ("cyberpunk-2077/night-city", "Night City"));
    assert_eq!(
        offers[0].thumbnail.as_deref(),
        Some("https://media.mapgenie.io/v2/assets/prod/games/cyberpunk-2077/maps/night-city/preview.webp")
    );
}

#[test]
#[ignore = "needs fixtures/mapgenie"]
fn a_map_page_without_map_cards_offers_its_own_map() {
    let offers = parse_game("cyberpunk-2077", &research!("pages/night-city.html"));
    assert_eq!(offers.len(), 1);
    assert_eq!((offers[0].id.as_str(), offers[0].name.as_str()), ("cyberpunk-2077/night-city", "Night City"));
}

#[test]
#[ignore = "needs fixtures/mapgenie"]
fn night_city_converts_with_tiles_pins_regions_and_links() {
    let (page, manifest, data) = (research!("pages/night-city.html"), research!("pages/icon-manifest.json"), research!("sample/data.json"));
    let map = night_city(&page, &manifest, &data);

    assert_eq!(map.tile_sets.len(), 2);
    assert_eq!((map.tile_sets[0].min_zoom, map.tile_sets[1].min_zoom), (10, 8));
    assert_eq!(map.tile_sets[0].template, "https://tiles.mapgenie.io/games/cyberpunk-2077/new/default-v2/{z}/{y}/{x}.jpg");
    for set in &map.tile_sets {
        let [top_left, bottom_right] = set.bounds.expect("bounds");
        for p in [top_left, bottom_right] {
            assert!((0. ..=1.).contains(&p.x) && (0. ..=1.).contains(&p.y), "{p:?}");
        }
        assert!(top_left.x < bottom_right.x && top_left.y < bottom_right.y);
    }

    let groups: Vec<&str> = map.categories.iter().flat_map(|c| &c.groups).map(|g| g.id.as_str()).collect();
    assert!(map.categories.iter().flat_map(|c| &c.groups).all(|g| g.icon.is_some()));
    let fast_travel = map.categories.iter().flat_map(|c| &c.groups).find(|g| g.id == "1724").expect("fast travel");
    assert!(matches!(&fast_travel.icon, Some(Icon::Image { cell: Some([384, 96, 96, 96]), .. })), "the glyph, not the whole marker");
    assert!(map.pins.len() > 2_500, "{}", map.pins.len());
    assert!(map.pins.iter().all(|p| groups.contains(&p.group.as_str())));
    assert!(map.pins.iter().all(|p| !p.description.contains("mapgenie.io/cyberpunk-2077/maps/night-city?locationIds=")));
    assert!(map.pins.iter().any(|p| p.description.contains(&format!("]({PIN_LINK}"))));
    assert!(map.pins.iter().any(|p| p.image.is_some()));

    assert!(!map.regions.is_empty() && map.regions.iter().all(|r| !r.outlines.is_empty()));
    assert!(map.regions.iter().any(|r| r.zoom.is_some()) && map.regions.iter().any(|r| r.zoom.is_none()));
    assert!(!map.labels.is_empty());
    assert!(serde_json::to_vec(&map).unwrap().len() < 16 * 1024 * 1024);
}

#[test]
fn the_findings_test_vector_projects_to_the_unit_square() {
    let p = mercator(0.78982289, -0.77955683);
    assert!((p.x - 0.49783456).abs() < 1e-6 && (p.y - 0.49780598).abs() < 1e-6, "{p:?}");
}

#[test]
#[ignore = "needs fixtures/mapgenie"]
fn a_link_to_another_map_or_a_bare_link_is_left_alone() {
    let data = r#"{"locations": [{"id": 1, "category_id": 1732, "title": "A", "latitude": "0.5", "longitude": "-0.5",
        "description": "[a](https://mapgenie.io/cyberpunk-2077/maps/night-city?locationIds=7) [b](https://mapgenie.io/other/maps/x?locationIds=8) https://mapgenie.io/cyberpunk-2077/maps/night-city?locationIds="}]}"#;
    let (page, manifest) = (research!("pages/night-city.html"), research!("pages/icon-manifest.json"));
    let map = night_city(&page, &manifest, data);
    assert_eq!(
        map.pins[0].description,
        "[a](pin:7) [b](https://mapgenie.io/other/maps/x?locationIds=8) https://mapgenie.io/cyberpunk-2077/maps/night-city?locationIds="
    );
}
