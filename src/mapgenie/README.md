# mapgenie

[MapGenie](https://mapgenie.io) as a provider of the plugin. The first one, so the reference for the next.

`mod.rs` is the provider's three calls, each over MapGenie's own ids (the plugin adds the `mapgenie/` prefix):

| call | id | source |
|---|---|---|
| `titles` | game slug | home page, `html::parse_home` |
| `maps` | game slug | game page, `html::parse_game` |
| `map` | `<game>/<map>` | map page (`page`), icon manifest and `/api/v1/maps/<id>/data` (`data`), joined by `convert` |

Files:
- `html.rs`, `page.rs` - scan the pages by hand: game cards, and the `window.*` objects of a map page's inline scripts.
- `data.rs` - typed shapes of the data endpoint and the icon manifest.
- `convert.rs` - all of it into the wire's `Map`.
- `stores.json` - game slug to launcher store ids (`steam:1091500`). Impulse links a library game to a title only through them, never by name.
- `tools/mapgenie-stores.py` - builds `stores.json` offline from a saved home page and the Wand catalog: `uv run src/mapgenie/tools/mapgenie-stores.py <home.html> <catalog.json>`.
- `tests.rs` - read saved pages from `fixtures/mapgenie/` (gitignored), so `#[ignore]`: `cargo test -- --include-ignored`.

Two providers may list one game, but only the first title that matches a game links on its own; the other is picked by hand.
