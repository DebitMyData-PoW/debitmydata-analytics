#!/usr/bin/env bash
# Refreshes this build's copy of DebitMyData's pool list (pools-v2.json, read when the server is given
# none: BITVIEW_POOLS) and the showcases' pool logos, from DebitMyData-PoW/mining-pools and
# DebitMyData-PoW/mining-pool-logos, as the server does every day (deploy/debitmydata/pools.sh).
set -euo pipefail

cd "$(dirname "$0")"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
DMD_POOLS_DIR="$WORK" ../../deploy/debitmydata/pools.sh
cp "$WORK/pools-v2.json" pools-v2.json
cp "$WORK"/site/assets/pools/*.svg ../../showcases/assets/pools/

# The pages draw only the logos assets/pools/logos.js lists, written here from the folder (a logo
# added by hand too: run again).
python3 - <<'PY'
import os, re, textwrap

ASSETS = "../../showcases/assets/pools"
files = sorted(name for name in os.listdir(ASSETS) if name.endswith(".svg"))
# Each list a line of slugs filled to the width before the next.
lists = {
    "LOGOS": [name[: -len(".svg")] for name in files if not name.endswith(".light.svg")],
    "LIGHT": [name[: -len(".light.svg")] for name in files if name.endswith(".light.svg")],
}
path = f"{ASSETS}/logos.js"
js = open(path).read()
for name, slugs in lists.items():
    items = textwrap.fill(" ".join(f'"{slug}",' for slug in slugs), 117, initial_indent="  ", subsequent_indent="  ")
    js, found = re.subn(rf"const {name} = new Set\(\[.*?\]\);", lambda _: f"const {name} = new Set([\n{items}\n]);", js, count=1, flags=re.S)
    assert found, f"{path}: no {name} list to write"
open(path, "w").write(js)
print(f"{path}: {len(files)} logo files")
PY
