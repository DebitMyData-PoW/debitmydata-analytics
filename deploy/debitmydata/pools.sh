#!/usr/bin/env bash
# DebitMyData's mining pools, kept up to date from GitHub: the list (DebitMyData-PoW/mining-pools,
# pools-v2.json) and the logos (DebitMyData-PoW/mining-pool-logos). Run every day by
# dmd-pools.timer, and by install.sh.
#
#   /opt/debitmydata-analytics/deploy/debitmydata/pools.sh
#
# Edit those two repositories and, within a day, the analytics site names the pools and shows
# their logos: bitviewd reads the list when it starts (BITVIEW_POOLS), so a changed list restarts
# it, and it attributes every block again; nginx serves the logos (/assets/pools/).
#
# Nothing from GitHub runs here: the list must read as a list of pools, each logo as a plain
# picture (no script, no link out, no embedded page). Anything else is left out, and what was
# there before stays.
set -euo pipefail

DIR="${DMD_POOLS_DIR:-/var/lib/debitmydata-analytics/pools}"
LIST=$DIR/pools-v2.json
LOGOS=$DIR/site/assets/pools
RAW=https://raw.githubusercontent.com/DebitMyData-PoW
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

mkdir -p "$LOGOS"
fetch() { curl -fsSL --proto '=https' --max-time 60 --max-filesize 3000000 "$1" -o "$2"; }

# The list.
fetch "$RAW/mining-pools/master/pools-v2.json" "$WORK/pools-v2.json"
python3 - "$WORK/pools-v2.json" <<'PY'
import json, re, sys
pools = json.load(open(sys.argv[1]))
assert isinstance(pools, list) and pools, "not a list of pools"
for pool in pools:
    assert isinstance(pool, dict), "a pool that isn't an object"
    assert isinstance(pool.get("id"), int) and 1 <= pool["id"] <= 255, f"{pool.get('name')}: id from 1 to 255"
    assert isinstance(pool.get("name"), str) and re.search(r"[A-Za-z0-9]", pool["name"]), "a pool without a name"
    for key in ("addresses", "tags"):
        assert all(isinstance(v, str) for v in pool.get(key, [])), f"{pool['name']}: {key} must be text"
    assert pool.get("country") is None or isinstance(pool["country"], str), f"{pool['name']}: country as text (US, or Turkey)"
PY

# The logos: each pool's (its name's letters and digits, as the analytics site writes it; the
# logos repository's file may keep the name's dashes), the default and the unknown pool's.
python3 - "$WORK/pools-v2.json" >"$WORK/wanted" <<'PY'
import json, re, sys
for pool in json.load(open(sys.argv[1])):
    name = pool["name"].lower()
    slug = re.sub(r"[^a-z0-9]", "", name)
    print(slug, re.sub(r"[^a-z0-9-]", "", name), slug)
print("default default default")
print("unknown unknown unknown")
PY
mkdir -p "$WORK/logos"
while read -r slug first second; do
    for name in "$first" "$second"; do
        if fetch "$RAW/mining-pool-logos/master/$name.svg" "$WORK/logos/$slug.svg" 2>/dev/null; then
            break
        fi
    done
done <"$WORK/wanted"
python3 - "$WORK/logos" <<'PY'
import os, re, sys
folder = sys.argv[1]
# (A picture, nothing more: no script, no handler, no link out, no page inside it.)
unsafe = re.compile(rb"<script|<foreignobject|<iframe|<embed|<object|javascript:|\bon[a-z]+\s*=|(?:xlink:)?href\s*=\s*[\"'](?!#|data:image/)", re.I)
for name in os.listdir(folder):
    path = os.path.join(folder, name)
    data = open(path, "rb").read()
    # (Its embedded pictures' bytes aside, which could spell anything.)
    bare = re.sub(rb"data:image/(?:png|jpeg|gif|webp);base64,[A-Za-z0-9+/=\s]*", b"data:image/", data)
    if b"<svg" not in data[:4096].lower() or unsafe.search(bare):
        print(f"left out: {name} (not a plain picture)", file=sys.stderr)
        os.remove(path)
PY
# (A heavy logo, a picture inside, made small: 96 pixels, WebP, inside an SVG, as the pages draw it.)
for file in "$WORK"/logos/*.svg; do
    [ -e "$file" ] || continue
    if [ "$(stat -c %s "$file")" -gt 150000 ] && command -v rsvg-convert >/dev/null &&
        rsvg-convert -w 96 -h 96 --keep-aspect-ratio "$file" -o "$WORK/small.png" 2>/dev/null; then
        # (WebP if cwebp is there, else the PNG.)
        small=$WORK/small.png kind=png
        if command -v cwebp >/dev/null && cwebp -quiet -q 88 "$WORK/small.png" -o "$WORK/small.webp" 2>/dev/null; then
            small=$WORK/small.webp kind=webp
        fi
        printf '<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 96 96"><image width="96" height="96" xlink:href="data:image/%s;base64,%s"/></svg>\n' \
            "$kind" "$(base64 -w0 "$small")" >"$file"
    fi
done
# What the pages draw: the logos there are (assets/pools/logos.js, as the site's own lists them).
python3 - "$WORK/logos" >"$WORK/logos/logos.js" <<'PY'
import os, sys
names = sorted(n[:-4] for n in os.listdir(sys.argv[1]) if n.endswith(".svg"))
print("// DebitMyData's pool logos, from DebitMyData-PoW/mining-pool-logos (deploy/debitmydata/pools.sh).")
print("export const LOGOS = new Set(%s);" % ("[" + ", ".join(f'"{n}"' for n in names) + "]"))
print("export const LIGHT = new Set([]);")
print("export const logo = (slug, light) => import.meta.resolve(`/assets/pools/${LOGOS.has(slug) ? slug : \"default\"}.svg`);")
PY

# Kept: logos first (a page asking for a new pool's finds it), then the list.
for file in "$WORK"/logos/*; do
    install -m 644 "$file" "$LOGOS/"
done
chmod 755 "$DIR" "$DIR/site" "$DIR/site/assets" "$LOGOS"
if cmp -s "$WORK/pools-v2.json" "$LIST" 2>/dev/null; then
    echo "pools: no change"
else
    install -m 644 "$WORK/pools-v2.json" "$LIST"
    echo "pools: the list changed; bitviewd restarts to read it"
    if systemctl is-active --quiet dmd-analytics 2>/dev/null; then
        systemctl restart dmd-analytics
    fi
fi
