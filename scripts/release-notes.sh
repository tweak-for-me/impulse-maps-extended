#!/usr/bin/env bash
# Fails unless a release tag `v<version>` is the manifest's version and CHANGELOG.md has its section; prints the section.
set -euo pipefail
cd "$(dirname "$0")/.."
version=$(jq -r .version manifest.json | tr -d '\r')
[ "${1:?usage: release-notes.sh <tag>}" = "v$version" ] || { echo "tag $1, but manifest.json says $version" >&2; exit 1; }
notes=$(awk -v head="## [$version]" '/^## \[/ { on = index($0, head) == 1; next } on' CHANGELOG.md | tr -d '\r')
[[ "$notes" =~ [^[:space:]] ]] || { echo "CHANGELOG.md has no section for $version" >&2; exit 1; }
echo "$notes"
