# DebitMyData Analytics

On-chain analytics for the DebitMyData (DMD) chain, at
[analytics.debitmydata.io](https://analytics.debitmydata.io): thousands of ready-made charts
(**Studio**), the network's hashrate, reference price and difficulty on one screen (**Oracle**),
every block, transaction and address (**Explorer**), where DMD stands against what its holders paid
(**Compass**), where the blocks are mined, by each pool's country (**Globe**), every DMD by the
price it last moved at, as a terrain (**Trail**), a plan buying DMD on a schedule
(**Stack**), each day's blocks and pools (**Calendar**), which pool first did each new thing on the
chain (**Miners**), every series browsed like files (**Catalog**), and every series, block and
address as data through an open **API**.

It is a fork of [Bitview](https://github.com/bitcoinresearchkit/mono), the Bitcoin Research Kit,
adapted to the DebitMyData chain. Bitview's own documentation is in [docs/](docs/README.md).

## What differs from Bitview

- **The chain.** DebitMyData's network (its block files' bytes), its addresses (`D…`, `dm1…`),
  its block reward (35 DMD, halving every 300,000 blocks, starting again after 32 halvings) and
  its 2.5-minute blocks, 576 a day. Bitcoin's two duplicate coinbases of 2010 are not DebitMyData's.
  All in one place: [`crates/brk_types/src/chain.rs`](crates/brk_types/src/chain.rs).
- **The price.** DMD has no market price yet. Bitview reads Bitcoin's price from round-dollar
  payments on its chain; here every block is priced with **DebitMyData's reference price**, the
  DebitMyData Agent Gateway's: the cost of the electricity it takes to mine one DMD (the chain's
  work over the last 4,032 blocks, at 20 J/TH and $0.06 per kWh, divided by the block reward), never
  below $0.001. Worked out from the chain alone, for every block since the first:
  [`crates/bitview_plugin_price/src/reference.rs`](crates/bitview_plugin_price/src/reference.rs).
  It is shown everywhere as the reference price, not a market price. Prices are counted in
  millionths of a dollar ($0.001 is a tenth of a cent; up to $4,294.97 on the charts of what
  holders paid). `GET /api/price` gives the latest.
- **Mining.** DebitMyData's mining pools instead of Bitcoin's: the list in
  [DebitMyData-PoW/mining-pools](https://github.com/DebitMyData-PoW/mining-pools) (`pools-v2.json`:
  each pool's number, name, the text it writes in its blocks, its addresses and its country) and
  the logos in [DebitMyData-PoW/mining-pool-logos](https://github.com/DebitMyData-PoW/mining-pool-logos),
  read from GitHub every day (see *The mining pools* below). The difficulty adjusts at every block
  (LWMA): Bitview's 2,016-block periods are kept for charts.
- **Bedrock** (the price floors Stack buys at) reads DebitMyData's own history of holders' losses:
  it has floors once the chain has 52,560 blocks behind it (about three months of DebitMyData's).
- **Left out.** Bitview's Rarity Meter model (calibrated on Bitcoin's history, 210,000 blocks from
  its first halving), its Ledger page and the Miners page's BIP-110 table (a Bitcoin proposal), its
  benchmarks, its MCP server and its plugin for
  assistants, and its terms, privacy and support pages (Bitview's own).
- **The Oracle.** Bitview's Oracle reads Bitcoin's price from round-dollar payments; here its design
  shows the network instead: hashrate (the default), the reference price or the difficulty, a
  switch at the top, over 7, 30 or 90 days or the whole chain. Its grid of squares behind the line
  is DebitMyData's blocks, each block's own value (its last hour's hashrate, its reference price or
  its difficulty), and its chapters below show how each is worked out, with small live charts.
- **The Explorer** draws its blocks full at 100,000 weight units (25,000 vB), as DebitMyData's
  mempool explorer does: its blocks are small, and a block drawn at Bitcoin's 4,000,000 would look
  empty. Each block says its size and transactions.
- **The website.** DebitMyData's name and colors, its coin with the analytics badge at the top
  left; the home page, Studio, Oracle, Explorer, Compass, Globe, Trail, Stack, Calendar, Miners,
  Catalog, the API's documentation and every series' description speak of DMD, its amounts in DMD
  and data (10^-8 DMD) and fee rates in data/vB. The API keeps Bitview's names (`btc`, `sats`,
  `cents`: DMD, data, millionths of a dollar), so that Bitview's clients and charts work unchanged.

## Checking it against a real node

[`scripts/debitmydata-regtest-check.py`](scripts/debitmydata-regtest-check.py) makes a private test
chain with the DebitMyData node (every address type, payments, a whole reward cycle), lets bitviewd
read it, and compares it with the node: every address's balance and coins, the coins in existence,
each block's reward, the addresses, the price and the market cap.

```
scripts/debitmydata-regtest-check.py --node-bin <folder with debitmydatad and debitmydata-cli> \
    --bitviewd target/debug/bitviewd
```

## What you need

- **A fresh Ubuntu 24.04 server**: 2 vCPU, 4 GB of memory, 80 GB of disk (Vultr: Cloud Compute,
  Regular Performance). SSH with your key. The install adds 8 GB of swap: building bitviewd needs
  more than 4 GB.
- **A name in Cloudflare**: `analytics.debitmydata.io`, pointing to the server's IP, **proxied**
  (orange cloud).
- **A Cloudflare origin certificate** for `*.debitmydata.io` and `debitmydata.io` (the one the
  monitoring server uses works: same names).
- **The DebitMyData Core binaries** (v1.3.5 or later): `debitmydatad` and `debitmydata-cli`,
  built for Linux x86_64, as for the monitoring server.

## Install

1. **The server.** Log in as root with your key, then turn off password logins:

   ```
   printf 'PasswordAuthentication no\nKbdInteractiveAuthentication no\nPermitRootLogin prohibit-password\n' > /etc/ssh/sshd_config.d/00-keys-only.conf
   sshd -t && systemctl reload ssh
   apt-get update && apt-get -y upgrade && apt-get -y install git
   ```

2. **The DebitMyData Core binaries.** Copy them from your computer (WinSCP, or `scp`) to
   `/usr/local/bin/`, then:

   ```
   chmod 755 /usr/local/bin/debitmydatad /usr/local/bin/debitmydata-cli
   ```

3. **The certificate.** Paste the origin certificate, then its private key (Ctrl+O, Enter, Ctrl+X
   to save):

   ```
   mkdir -p /etc/ssl/cloudflare
   nano /etc/ssl/cloudflare/debitmydata.io.pem
   nano /etc/ssl/cloudflare/debitmydata.io.key
   chmod 600 /etc/ssl/cloudflare/debitmydata.io.key
   ```

4. **This repository, and the install.** The first time it builds bitviewd: an hour or more on
   2 vCPU (it prints how long it took). Run it in `tmux` so that a dropped connection does not stop
   it (`tmux attach` to come back):

   ```
   git clone https://github.com/DebitMyData-PoW/debitmydata-analytics.git /opt/debitmydata-analytics
   apt-get -y install tmux && tmux
   ```

   A green bar appears at the bottom: you are in tmux. Then, on its own:

   ```
   /opt/debitmydata-analytics/deploy/debitmydata/install.sh
   ```

   It asks two questions; Enter keeps the answer in brackets (`analytics.debitmydata.io`,
   `debitmydata.io` for the certificate's file names).

5. **Check.** The node syncs the chain first, then bitviewd reads every block (a few minutes for
   DebitMyData's chain); then `https://analytics.debitmydata.io` answers. Follow it:

   ```
   journalctl -fu dmd-analytics
   ```

## Commands

```
dmd-cli getblockchaininfo                  # the node
systemctl status dmd-analytics             # bitviewd
journalctl -u dmd-analytics -n 50          # its log
systemctl restart dmd-analytics
curl -s https://analytics.debitmydata.io/api/price   # the reference price now
dmd-tor-check --status                     # Tor: running, its version, connected, the node through it
journalctl -u dmd-tor-check                # what the Tor check found and repaired
sudo systemctl start dmd-pools             # the mining pools and their logos from GitHub now
journalctl -u dmd-pools                    # what it read
```

## Updating

```
cd /opt/debitmydata-analytics && git pull && deploy/debitmydata/install.sh
```

It builds bitviewd again only when the repository changed (only what changed is compiled again),
and keeps the answers, the node's chain and bitviewd's data. When a new version stores its data
differently, bitviewd reads the chain again by itself.

When the gateway's price settings change (electricity, machines' efficiency, the minimum rate),
add an entry from that date in `crates/bitview_plugin_price/src/reference.rs` (`SETTINGS`): the
blocks before it keep their prices.

## The mining pools

The pools come from two GitHub repositories, read every day by `dmd-pools.timer`
([`deploy/debitmydata/pools.sh`](deploy/debitmydata/pools.sh)):

- **The list**, [DebitMyData-PoW/mining-pools](https://github.com/DebitMyData-PoW/mining-pools)
  (`pools-v2.json`). Each pool: `id` (1 to 255, never changed once given), `name`, `tags` (text its
  blocks' coinbase holds), `addresses` (where its blocks pay), `link` and `country` (two letters,
  `US`, or the country's English name, `Turkey`: Globe places the pool there). A block is the first
  pool, by number, whose tag its coinbase holds or whose address it pays. When the list changes,
  bitviewd restarts and attributes every block again (a few minutes).
- **The logos**, [DebitMyData-PoW/mining-pool-logos](https://github.com/DebitMyData-PoW/mining-pool-logos):
  one SVG per pool, named as its name in lowercase letters and digits (`Coin-Miners`:
  `coinminers.svg`; `coin-miners.svg` is read too), and `default.svg` for a pool without one.
  Only plain pictures are kept (no script, no link out); a heavy one is made small (96 pixels).
  nginx serves them at `/assets/pools/`.

Nothing else from GitHub reaches the server. A pool's slug (in its address on the site and its
series' names, `pool_coinminers_…`) is its name in lowercase letters and digits, so renaming a pool
renames its series. The server answers for the current list at once, and so do Globe, the Explorer,
the Calendar, the home page and Studio, whose charts of each pool include a pool added since (no
rebuild). The generated API clients list the pools by name as they were when they were made
(`BITVIEW_POOLS=<the list> cargo run -p bitview_devtools --bin bitview-bindgen`); `poolSeries(slug)`
reaches any pool's series.

## After the first sync (optional)

Studio's list of charts for search engines (`showcases/studio/sitemap.xml`) is made from the live
server. On a computer with [Bun](https://bun.sh), in this repository:

```
bun showcases/studio/seo.mjs https://analytics.debitmydata.io
```

Then commit the changed files and update the server.

The home page's pictures are screenshots of the apps, as Bitview's. Made again from the live site
(with a Chromium installed):

```
bun showcases/home.mjs --pictures https://analytics.debitmydata.io
```

## Security

- The server holds **no keys and no coins**: its node has no wallet, and its RPC answers this
  server only. bitviewd reads the node's block files (read only: the node's folder gives its group
  read access, never write) and asks the node over RPC.
- **The node's IP address stays hidden too.** The node accepts no connections (`listen=0`) and
  reaches its peers through **Tor** ([`tor.sh`](deploy/debitmydata/tor.sh): `proxy=127.0.0.1:9050`),
  so its peers, and the explorers that list the peers they see, only get a Tor exit's address.
  Tor comes from the Tor Project's own repository; unattended-upgrades installs each new version
  within a day, and `dmd-tor-check` (every 30 minutes) restarts Tor when it stops or is cut off,
  and upgrades it at once when the Tor network says its version is too old. Its problems go to its
  log, and to Discord with a webhook in `/etc/default/dmd-tor-check` (`DMD_TOR_WEBHOOK=`).
- **Only Cloudflare reaches ports 80 and 443** ([`cloudflare.sh`](deploy/debitmydata/cloudflare.sh);
  run it again if Cloudflare changes its addresses). bitviewd listens on this server only, behind
  nginx, which limits each visitor's requests and keeps the server's own log private.
- Nothing secret is in this repository.

## License

MIT, as Bitview: [LICENSE](LICENSE). Bitview is © its authors (Bitcoin Research Kit & Bitview);
the DebitMyData changes are © DebitMyData.
