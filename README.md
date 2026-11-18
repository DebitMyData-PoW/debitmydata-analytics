# DebitMyData Analytics

On-chain analytics for the DebitMyData (DMD) chain, at
[analytics.debitmydata.io](https://analytics.debitmydata.io): thousands of ready-made charts
(**Studio**), every series browsed like files (**Series**), each day's blocks and pools
(**Calendar**), and every series, block and address as data through an open **API**.

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
- **Mining.** DebitMyData's pool (its coinbase text, `Official`) instead of Bitcoin's pools; the
  difficulty adjusts at every block (LWMA): Bitview's 2,016-block periods are kept for charts.
- **Left out.** Bitview's Bedrock and Rarity Meter models (calibrated on Bitcoin's market history),
  and its Bitcoin-only pages (Oracle, Compass, Stack, Globe, the Satoshi Trail).
- **The website.** DebitMyData's name, icon and colors; the home page, Studio, Series, Calendar,
  the API's documentation and every series' description speak of DMD. The API keeps Bitview's
  names (`btc`, `sats`, `cents`: DMD, its 10^-8 unit, millionths of a dollar), so that Bitview's
  clients and charts work unchanged.

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

## After the first sync (optional)

Studio's list of charts for search engines (`showcases/studio/sitemap.xml`) is made from the live
server. On a computer with [Bun](https://bun.sh), in this repository:

```
bun showcases/studio/seo.mjs https://analytics.debitmydata.io
```

Then commit the changed files and update the server.

The home page's pictures are DebitMyData's own: the coin with the microscope badge, over a motif
for each app. `bun showcases/home.mjs --pictures https://analytics.debitmydata.io` would put
screenshots of the apps in their place (the coin then only on the share card); leave it unless
you want that.

## Security

- The server holds **no keys and no coins**: its node has no wallet, and its RPC answers this
  server only. bitviewd reads the node's block files (read only: the node's folder gives its group
  read access, never write) and asks the node over RPC.
- **Only Cloudflare reaches ports 80 and 443** ([`cloudflare.sh`](deploy/debitmydata/cloudflare.sh);
  run it again if Cloudflare changes its addresses). bitviewd listens on this server only, behind
  nginx, which limits each visitor's requests and keeps the server's own log private.
- Nothing secret is in this repository.

## License

MIT, as Bitview: [LICENSE](LICENSE). Bitview is © its authors (Bitcoin Research Kit & Bitview);
the DebitMyData changes are © DebitMyData.
