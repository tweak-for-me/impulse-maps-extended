# Impulse Maps Extended

A plugin for [Impulse](https://github.com/tweak-for-me/impulse) that adds new interactive maps for games.

Maps come from providers:
- [MapGenie](src/mapgenie/README.md)

## Install

Take `maps-extended-<version>.zip` from the Releases and install it in Impulse. Impulse asks you to approve the hosts the plugin talks to (`hosts` in `manifest.json`).

## Development

A WASM plugin built against Impulse's `sdk`.

- `src/lib.rs` - the plugin: `SITES`, one row per provider, and the routing. The host's ids are `<provider>/<the provider's id>`, so a provider's module never sees its prefix.
- `src/<provider>/` - one provider: `titles`, `maps` and `map` over its own ids, its data and tests.
- `src/kit/` - what any provider needs: text scanning, `get_text`, the common errors. A helper moves here when a second provider needs it.
- `manifest.json` - the one manifest: `hosts` is every provider's hosts, which the user approves at install.
- `fixtures/<provider>/` - a provider's saved pages and data for native tests. Gitignored, so the tests that read them are `#[ignore]`.

```sh
cargo test -- --include-ignored   # native tests, fixtures in place
bash scripts/build.sh             # dist/maps-extended/ and dist/maps-extended-<version>.zip: what Impulse installs (needs jq, python3)
```

A new provider: a module in `src/` with the three calls, a row in `SITES`, its hosts in `manifest.json`.

Release: raise `version` in `manifest.json` and `Cargo.toml`, add its section to `CHANGELOG.md`, commit, then `git tag v<version>` and push the tag. `.github/workflows/release.yml` refuses a tag that differs from the manifest or has no changelog section, tests, builds and publishes the zip with that section as the release notes.
