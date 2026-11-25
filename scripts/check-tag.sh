#!/usr/bin/env bash
# Fails unless a release tag `v<version>` is the manifest's version.
set -euo pipefail
version=$(jq -r .version "$(dirname "$0")/../manifest.json" | tr -d '\r')
[ "${1:?usage: check-tag.sh <tag>}" = "v$version" ] || { echo "tag $1, but manifest.json says $version" >&2; exit 1; }
