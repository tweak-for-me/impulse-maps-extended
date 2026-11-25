"""Builds src/mapgenie/stores.json: each MapGenie game slug with the store ids of the matching title in the local Wand
catalog (by slug, then by name, then ALIASES). Offline: reads a saved home page and the catalog the app cached.

    uv run src/mapgenie/tools/mapgenie-stores.py <home.html> <catalog.json>
"""

import html
import json
import re
import sys
from pathlib import Path

# MapGenie slug -> Wand title slugs, where names differ.
ALIASES = {
    "metal-gear-solid-5-the-phantom-pain": ["metal-gear-solid-v-tpp"],
    "oblivion": ["the-elder-scrolls-iv-oblivion-game-of-the-year-edition-deluxe"],
    "death-stranding-2": ["death-stranding-2-on-the-beach"],
    "resident-evil-4-remake": ["resident-evil-4"],
    "horizon-forbidden-west": ["horizon-forbidden-west-complete-edition"],
    "dying-light-2": ["dying-light-2-stay-human"],
    "ghost-of-tsushima": ["ghost-of-tsushima-directors-cut"],
    "star-wars-jedi-fallen-order": ["star-wars-jedi-fallen-order-tm"],
    "marvels-spider-man": ["marvels-spider-man-remastered"],
    "resident-evil-2-remake": ["resident-evil-2-biohazard-re-2"],
    "god-of-war-2018": ["god-of-war"],
    "horizon-zero-dawn": ["horizon-zero-dawn-tm-complete-edition", "horizon-zero-dawn-remastered"],
    "witcher-3": ["the-witcher-3", "the-witcher-3-wild-hunt-game-of-the-year-edition-game"],
    "assassins-creed-3-liberation": ["assassins-creed-liberation-hd"],
    "skyrim": ["the-elder-scrolls-v-skyrim", "the-elder-scrolls-v-skyrim-special-edition"],
    "fallout-new-vegas": ["fallout-new-vegas-ultimate-edition"],
    "batman-arkham-asylum": ["batman-arkham-asylum-game-of-the-year-edition"],
    "red-faction-guerrilla": ["red-faction-guerrilla-steam-edition", "red-faction-guerrilla-re-mars-tered"],
    "resident-evil-5": ["resident-evil-5-biohazard-5"],
    "bully": ["bully-scholarship-edition"],
}

CARD = re.compile(r'data-game-id="\d+".*?href="https://mapgenie\.io/([^"/]+)".*?<h4 class="card-title">([^<]*)</h4>', re.S)


def norm(text):
    text = html.unescape(text).lower().replace("&", "and")
    text = re.sub(r"[^a-z0-9]+", " ", text)
    return " ".join(word for word in text.split() if word not in ("the", "a", "an"))


def main(home, catalog):
    catalog = json.loads(Path(catalog).read_text(encoding="utf-8"))
    titles = list(catalog["titles"].values())
    by_slug = {title["slug"]: title for title in titles}
    by_name = {}
    for title in titles:
        by_name.setdefault(norm(title["name"]), title)

    def stores(title):
        found = []
        for game in filter(None, map(catalog["games"].get, title["gameIds"])):
            for store in game.get("correlationIds", []):
                if not store.lower().startswith("standalone:") and store not in found:
                    found.append(store)
        return found

    table = {}
    for slug, name in CARD.findall(Path(home).read_text(encoding="utf-8")):
        matches = [by_slug[alias] for alias in ALIASES.get(slug, []) if alias in by_slug]
        matches = matches or [title for title in (by_slug.get(slug) or by_name.get(norm(name)),) if title]
        ids = [store for title in matches for store in stores(title)]
        if ids:
            table[slug] = sorted(set(ids))
    out = Path(__file__).resolve().parent.parent / "stores.json"
    out.write_text(json.dumps(dict(sorted(table.items())), indent=1) + "\n", encoding="utf-8", newline="\n")
    print(f"{len(table)} games with store ids -> {out}")


if __name__ == "__main__":
    main(*sys.argv[1:3])
