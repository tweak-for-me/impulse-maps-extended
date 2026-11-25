//! MapGenie's page, icon manifest and data into the wire's `Map`.

use std::borrow::Cow;
use std::collections::HashSet;

use sdk::guest::mercator;
use sdk::wire::{Category, Error, Group, Icon, Label, Map, PIN_LINK, Picture, Pin, Point, Region, TileSet};

use super::data::{Data, Geometry, IconManifest, RegionData};
use super::page::{GroupData, Page};

const WHITE: u32 = 0xFF_FF_FF;
/// Child regions are small: they show once the user is close.
const CHILD_ZOOM: [f32; 2] = [13., 20.];

pub fn convert(page: Page, manifest: &IconManifest, data: Data) -> Result<Map, Error> {
    let bounds = data
        .styles
        .as_ref()
        .and_then(|s| s.map_style.as_ref())
        .and_then(|s| s.bounds)
        .map(|[west, south, east, north]| [mercator(north, west), mercator(south, east)]);
    let mut tile_sets = page.tile_sets.iter().collect::<Vec<_>>();
    tile_sets.sort_by(|a, b| a.order.total_cmp(&b.order));
    let tile_sets = tile_sets
        .into_iter()
        .map(|set| TileSet {
            id: set.id.to_string(),
            name: set.name.clone(),
            template: format!("{}{}", page.tiles_cdn, set.pattern),
            flipped: false,
            size: 256,
            min_zoom: set.min_zoom,
            max_zoom: set.tiles_max_zoom,
            bounds,
        })
        .collect();

    let mut groups = page.groups.iter().collect::<Vec<_>>();
    groups.sort_by(|a, b| a.order.total_cmp(&b.order));
    let categories: Vec<Category> = groups.into_iter().map(|group| category(group, manifest)).collect();
    let known: HashSet<u64> = page.groups.iter().flat_map(|g| g.categories.iter().map(|c| c.id)).collect();

    let link = format!("https://mapgenie.io/{}/maps/{}?locationIds=", page.game_slug, page.map.slug);
    let pins = data
        .locations
        .iter()
        .filter(|location| known.contains(&location.category_id))
        .filter_map(|location| {
            let at = mercator(location.latitude?, location.longitude?);
            let image =
                location.media.iter().find(|m| m.kind == "image").map(|m| Picture { preview: m.url.to_string(), full: m.url.to_string() });
            Some(Pin {
                id: location.id.to_string(),
                at,
                name: location.title.to_string(),
                description: rewrite_links(location.description.as_deref().unwrap_or_default(), &link).into_owned(),
                group: location.category_id.to_string(),
                image,
                video: None,
            })
        })
        .collect();

    let regions: Vec<(&RegionData, Vec<Vec<Point>>)> =
        data.regions.iter().map(|region| (region, outlines(region))).filter(|(_, outlines)| !outlines.is_empty()).collect();
    let labels = regions
        .iter()
        .filter(|(region, _)| region.parent_region_id.is_none())
        .filter_map(|(region, outlines)| label(region, outlines))
        .collect();
    let regions = regions
        .into_iter()
        .map(|(region, outlines)| Region {
            id: region.id.to_string(),
            name: region.title.to_string(),
            color: WHITE,
            thickness: 1.5,
            outlines,
            zoom: region.parent_region_id.map(|_| CHILD_ZOOM),
        })
        .collect();

    Ok(Map { tile_sets, background: 0x10_10_10, categories, pins, regions, labels, unfinished: false })
}

fn category(group: &GroupData, manifest: &IconManifest) -> Category {
    let mut categories = group.categories.iter().collect::<Vec<_>>();
    categories.sort_by(|a, b| a.order.total_cmp(&b.order));
    Category {
        name: group.title.clone(),
        color: group.color.as_deref().and_then(|hex| u32::from_str_radix(hex.trim_start_matches('#'), 16).ok()).unwrap_or(WHITE),
        shown: true,
        groups: categories.into_iter().map(|c| Group { id: c.id.to_string(), name: c.title.clone(), icon: icon(manifest, c.id) }).collect(),
    }
}

fn icon(manifest: &IconManifest, category: u64) -> Option<Icon> {
    let own = manifest.categories.get(&category.to_string()).and_then(|c| c.glyph.as_deref());
    let fallback = manifest.fallback.as_ref().and_then(|c| c.glyph.as_deref());
    let cell = own.and_then(|name| manifest.glyphs.get(name)).or_else(|| fallback.and_then(|name| manifest.glyphs.get(name)))?;
    let atlas = manifest.atlas.as_ref()?;
    Some(Icon::Image { url: atlas.url.clone(), cell: Some([cell.x, cell.y, cell.w, cell.h]) })
}

/// `link` followed by a pin id becomes the host's pin link; any other text stays.
fn rewrite_links<'a>(text: &'a str, link: &str) -> Cow<'a, str> {
    if !text.contains(link) {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(link) {
        out.push_str(&rest[..at]);
        let after = &rest[at + link.len()..];
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            out.push_str(link);
        } else {
            out.push_str(PIN_LINK);
            out.push_str(&after[..digits]);
        }
        rest = &after[digits..];
    }
    out.push_str(rest);
    Cow::Owned(out)
}

/// The outer ring of every polygon of the region, holes dropped.
fn outlines(region: &RegionData) -> Vec<Vec<Point>> {
    let project = |ring: &Vec<[f64; 2]>| ring.iter().map(|&[lng, lat]| mercator(lat, lng)).collect::<Vec<_>>();
    let mut rings = Vec::new();
    for geometry in region.features.iter().filter_map(|f| f.geometry.as_ref()) {
        match geometry {
            Geometry::Polygon { coordinates } => rings.extend(coordinates.first().map(project)),
            Geometry::MultiPolygon { coordinates } => rings.extend(coordinates.iter().filter_map(|polygon| polygon.first()).map(project)),
            Geometry::Other => {}
        }
    }
    rings.retain(|ring| !ring.is_empty());
    rings
}

fn label(region: &RegionData, outlines: &[Vec<Point>]) -> Option<Label> {
    let mut points = outlines.iter().flatten();
    let first = points.next()?;
    let (min, max) = points.fold(((first.x, first.y), (first.x, first.y)), |(min, max), p| {
        ((min.0.min(p.x), min.1.min(p.y)), (max.0.max(p.x), max.1.max(p.y)))
    });
    Some(Label {
        text: region.title.to_string(),
        color: WHITE,
        size: 13.,
        at: Point { x: (min.0 + max.0) / 2., y: (min.1 + max.1) / 2. },
        zoom: None,
    })
}
