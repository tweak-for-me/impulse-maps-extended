//! Maps Extended: the maps of several sites behind one plugin. A site is a module with three calls over its own ids;
//! the host's ids are `<site>/<the site's id>`.

mod kit;
mod mapgenie;

use sdk::guest::{Context, Provider};
use sdk::wire::{Error, Map, MapOffer, Title};

use crate::kit::not_found;

struct Site {
    id: &'static str,
    titles: fn(&Context) -> Result<Vec<Title>, Error>,
    maps: fn(&Context, &str) -> Result<Vec<MapOffer>, Error>,
    map: fn(&Context, &str) -> Result<Map, Error>,
}

const SITES: &[Site] = &[Site { id: "mapgenie", titles: mapgenie::titles, maps: mapgenie::maps, map: mapgenie::map }];

struct MapsExtended;

sdk::export!(MapsExtended);

impl Provider for MapsExtended {
    /// All or nothing: a list without a failed site would be kept as the whole one until it is stale.
    fn titles(context: &Context) -> Result<Vec<Title>, Error> {
        let mut all = Vec::new();
        for site in SITES {
            all.extend((site.titles)(context)?.into_iter().map(|title| Title { id: prefixed(site, &title.id), ..title }));
        }
        Ok(all)
    }

    fn maps(context: &Context, title: &str) -> Result<Vec<MapOffer>, Error> {
        let (site, id) = route(title)?;
        Ok((site.maps)(context, id)?.into_iter().map(|offer| MapOffer { id: prefixed(site, &offer.id), ..offer }).collect())
    }

    fn map(context: &Context, map: &str) -> Result<Map, Error> {
        let (site, id) = route(map)?;
        (site.map)(context, id)
    }
}

fn prefixed(site: &Site, id: &str) -> String {
    format!("{}/{id}", site.id)
}

fn route(id: &str) -> Result<(&'static Site, &str), Error> {
    let (name, rest) = id.split_once('/').ok_or_else(|| not_found(id))?;
    let site = SITES.iter().find(|site| site.id == name).ok_or_else(|| not_found(id))?;
    Ok((site, rest))
}

#[cfg(test)]
mod tests {
    use sdk::wire::ErrorKind;

    use super::*;

    #[test]
    fn an_id_goes_to_its_site_with_the_rest_untouched() {
        let (site, rest) = route("mapgenie/cyberpunk-2077/night-city").unwrap();
        assert_eq!((site.id, rest), ("mapgenie", "cyberpunk-2077/night-city"));
        assert_eq!(prefixed(site, rest), "mapgenie/cyberpunk-2077/night-city");
    }

    #[test]
    fn an_id_without_a_known_site_is_not_found() {
        for id in ["cyberpunk-2077", "nowhere/x", ""] {
            assert_eq!(route(id).err().map(|e| e.kind), Some(ErrorKind::NotFound), "{id:?}");
        }
    }
}
