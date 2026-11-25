//! The typed shapes of `GET /api/v1/maps/<id>/data` and of the icon manifest; strings borrow from the body.

use std::borrow::Cow;
use std::collections::HashMap;

use serde::Deserialize;
use serde::de::{self, Deserializer, Visitor};

#[derive(Deserialize)]
pub struct Data<'a> {
    #[serde(borrow, default)]
    pub regions: Vec<RegionData<'a>>,
    #[serde(borrow, default)]
    pub locations: Vec<Location<'a>>,
    #[serde(default)]
    pub styles: Option<Styles>,
}

#[derive(Deserialize)]
pub struct Styles {
    #[serde(rename = "mapStyle", default)]
    pub map_style: Option<MapStyle>,
}

#[derive(Deserialize)]
pub struct MapStyle {
    /// West, south, east, north, in degrees.
    #[serde(default)]
    pub bounds: Option<[f64; 4]>,
}

#[derive(Deserialize)]
pub struct RegionData<'a> {
    pub id: u64,
    #[serde(default)]
    pub parent_region_id: Option<u64>,
    #[serde(borrow)]
    pub title: Cow<'a, str>,
    #[serde(default)]
    pub features: Vec<Feature>,
}

#[derive(Deserialize)]
pub struct Feature {
    #[serde(default)]
    pub geometry: Option<Geometry>,
}

/// Positions are `[lng, lat]`; the first ring of a polygon is its outer one.
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Geometry {
    Polygon {
        coordinates: Vec<Vec<[f64; 2]>>,
    },
    MultiPolygon {
        coordinates: Vec<Vec<Vec<[f64; 2]>>>,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
pub struct Location<'a> {
    pub id: u64,
    pub category_id: u64,
    #[serde(borrow)]
    pub title: Cow<'a, str>,
    #[serde(borrow, default)]
    pub description: Option<Cow<'a, str>>,
    #[serde(default, deserialize_with = "number")]
    pub latitude: Option<f64>,
    #[serde(default, deserialize_with = "number")]
    pub longitude: Option<f64>,
    #[serde(borrow, default)]
    pub media: Vec<Media<'a>>,
}

#[derive(Deserialize)]
pub struct Media<'a> {
    #[serde(borrow)]
    pub url: Cow<'a, str>,
    #[serde(rename = "type", borrow)]
    pub kind: Cow<'a, str>,
}

/// Coordinates come as decimal strings, a number elsewhere: both read.
fn number<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<f64>, D::Error> {
    struct Any;

    impl Visitor<'_> for Any {
        type Value = Option<f64>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a number, a numeric string or null")
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
            Ok(Some(v))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(v as f64))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v as f64))
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            v.trim().parse().map(Some).map_err(E::custom)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
    }

    deserializer.deserialize_any(Any)
}

/// `icon-set/build/v2/manifest.json`: a sprite sheet and the cell of each glyph.
#[derive(Deserialize, Default)]
pub struct IconManifest {
    #[serde(default)]
    pub atlas: Option<Atlas>,
    /// Category id (as text) to its glyph.
    #[serde(default)]
    pub categories: HashMap<String, CategoryIcon>,
    /// The bare symbols; `markers` are whole pins, which the host draws itself.
    #[serde(default)]
    pub glyphs: HashMap<String, Cell>,
    #[serde(default)]
    pub fallback: Option<CategoryIcon>,
}

#[derive(Deserialize)]
pub struct Atlas {
    pub url: String,
}

#[derive(Deserialize)]
pub struct CategoryIcon {
    #[serde(default)]
    pub glyph: Option<String>,
}

#[derive(Deserialize, Clone, Copy)]
pub struct Cell {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}
