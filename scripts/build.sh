#!/usr/bin/env bash
# Builds the plugin into dist/<id>/ and dist/<id>-<version>.zip: the wasm beside its manifest, what Impulse installs.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo build --release --target wasm32-unknown-unknown
read -r id module version < <(jq -r '[.id, .module, .version] | @tsv' manifest.json | tr -d '\r')

rm -rf dist
mkdir -p "dist/$id"
cp "target/wasm32-unknown-unknown/release/$module" manifest.json "dist/$id"
if [ -d locales ]; then cp -r locales "dist/$id"; fi
(cd "dist/$id" && python3 -m zipfile -c "../$id-$version.zip" *)
