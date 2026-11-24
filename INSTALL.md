# Installing DebitMyData Analytics

These steps install the analytics on a server of your own: a DebitMyData node, bitviewd (the
analytics server) and the website behind nginx. Below, `example.com` and `analytics.example.com`
stand for your own domain and name.

## What you need

- **A fresh Ubuntu 24.04 server** (x86_64): 2 vCPU, 4 GB of memory, 80 GB of disk, at any
  provider. Root access with an SSH key. The install adds 8 GB of swap: building bitviewd needs
  more than 4 GB.
- **A name** for the website (for example `analytics.example.com`), as a DNS record in
  **Cloudflare**, **proxied** (the orange cloud): the install lets only Cloudflare reach the web
  ports.
- **A Cloudflare origin certificate** for that name: in Cloudflare, your domain, **SSL/TLS**,
  **Origin Server**, **Create Certificate** (RSA, 15 years). The names Cloudflare suggests,
  `example.com` and `*.example.com`, cover it. In **SSL/TLS**, **Overview**, set the encryption
  mode to **Full (strict)**.
- **DebitMyData Core** 1.3.5 or later, for Linux x86_64 (step 2).

## Install

1. **The server.** Log in as root with your key, then turn off password logins and update it:

   ```
   printf 'PasswordAuthentication no\nKbdInteractiveAuthentication no\nPermitRootLogin prohibit-password\n' > /etc/ssh/sshd_config.d/00-keys-only.conf
   sshd -t && systemctl reload ssh
   apt-get update && apt-get -y upgrade && apt-get -y install git
   ```

2. **DebitMyData Core.** Download the Linux release from
   [GitHub](https://github.com/DebitMyData-PoW/DebitMyData-PoW-Core/releases) and check it: the
   SHA-256 that `sha256sum` prints must be the one published for that release. For a newer
   version, change `1.3.5` everywhere below:

   ```
   cd /tmp
   curl -LO https://github.com/DebitMyData-PoW/DebitMyData-PoW-Core/releases/download/v1.3.5/debitmydata-1.3.5-x86_64-linux-gnu.tar.gz
   sha256sum debitmydata-1.3.5-x86_64-linux-gnu.tar.gz
   tar -xzf debitmydata-1.3.5-x86_64-linux-gnu.tar.gz
   install -m 755 debitmydata-1.3.5-x86_64-linux-gnu/debitmydatad debitmydata-1.3.5-x86_64-linux-gnu/debitmydata-cli /usr/local/bin/
   rm -rf debitmydata-1.3.5-x86_64-linux-gnu*
   ```

   Binaries built from [the source](https://github.com/DebitMyData-PoW/DebitMyData-PoW-Core) work
   too: copy `debitmydatad` and `debitmydata-cli` to `/usr/local/bin/`.

3. **The certificate.** Paste the origin certificate, then its private key (in nano: Ctrl+O,
   Enter, Ctrl+X to save). Name both files after your domain:

   ```
   mkdir -p /etc/ssl/cloudflare
   nano /etc/ssl/cloudflare/example.com.pem
   nano /etc/ssl/cloudflare/example.com.key
   chmod 600 /etc/ssl/cloudflare/example.com.key
   ```

4. **This repository, and the install.** The first time, it builds bitviewd: an hour or more on
   2 vCPU (it prints how long it took). Run it in `tmux`, so that a dropped connection does not
   stop it (`tmux attach` to come back):

   ```
   git clone https://github.com/DebitMyData-PoW/debitmydata-analytics.git /opt/debitmydata-analytics
   apt-get -y install tmux && tmux
   ```

   A green bar appears at the bottom: you are in tmux. Then, on its own:

   ```
   /opt/debitmydata-analytics/deploy/debitmydata/install.sh
   ```

   It asks two questions: the website's name (`analytics.example.com`) and the certificate's name
   from step 3 (`example.com`). Type your own: Enter keeps the answer in brackets.

5. **Check.** The node syncs the chain first, then bitviewd reads every block (a few minutes for
   the DebitMyData chain today); then `https://analytics.example.com` answers. Follow it with:

   ```
   journalctl -fu dmd-analytics
   ```

6. **Tor's alerts** (optional). The node reaches its peers through Tor, and `dmd-tor-check` repairs
   Tor every 30 minutes. To also get its problems on Discord, put a channel's webhook in
   `/etc/default/dmd-tor-check` (`DMD_TOR_WEBHOOK=`).

## Commands

```
dmd-cli getblockchaininfo                  # the node
systemctl status dmd-analytics             # bitviewd
journalctl -u dmd-analytics -n 50          # its log
systemctl restart dmd-analytics
curl -s https://analytics.example.com/api/price   # the reference price now
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

**The reference price's settings.** When the electricity price, the machines' efficiency or the
minimum rate change, add an entry from that date in
`crates/bitview_plugin_price/src/reference.rs` (`SETTINGS`), commit it and update the server: the
blocks before that date keep their prices.

## The mining pools

The pools come from two GitHub repositories, read every day by `dmd-pools.timer`
([`deploy/debitmydata/pools.sh`](deploy/debitmydata/pools.sh), where their addresses are):

- **The list**, [DebitMyData-PoW/mining-pools](https://github.com/DebitMyData-PoW/mining-pools)
  (`pools-v2.json`). Each pool: `id` (1 to 255, never changed once given), `name`, `tags` (text its
  blocks' coinbase holds), `addresses` (where its blocks pay), `link` and `country` (two letters,
  `US`, or the country's English name, `Turkey`: Globe places the pool there). A block belongs to
  the first pool, by number, whose tag its coinbase holds or whose address it pays. When the list
  changes, bitviewd restarts and attributes every block again (a few minutes).
- **The logos**, [DebitMyData-PoW/mining-pool-logos](https://github.com/DebitMyData-PoW/mining-pool-logos):
  one SVG per pool, named as its name in lowercase letters and digits (`Coin-Miners`:
  `coinminers.svg`; `coin-miners.svg` is read too), and `default.svg` for a pool without one.
  Only plain pictures are kept (no script, no link out); a heavy one is made small (96 pixels).
  nginx serves them at `/assets/pools/`.

Nothing else from GitHub reaches the server. A pool's slug (in its address on the site and its
series' names, `pool_coinminers_…`) is its name in lowercase letters and digits, so renaming a pool
renames its series. The server, Globe, the Explorer, the Calendar, the home page and Studio follow
the current list at once, without a rebuild. The generated API clients list the pools by name as
they were when they were made
(`BITVIEW_POOLS=<the list> cargo run -p bitview_devtools --bin bitview-bindgen`); `poolSeries(slug)`
reaches any pool's series.

## After the first sync (optional)

Studio's list of charts for search engines (`showcases/studio/sitemap.xml`) is made from the live
server. On a computer with [Bun](https://bun.sh), in this repository:

```
bun showcases/studio/seo.mjs https://analytics.example.com
```

Then commit the changed files and update the server.

The home page's pictures are screenshots of the apps: each app dark, and on paper for the light
theme (`NAME-light.jpg`; until those are made, the light theme shows the dark ones). To make them
again from the live site (with a Chromium installed), then commit them:

```
bun showcases/home.mjs --pictures https://analytics.example.com
```

## Checking it against a real node

[`scripts/debitmydata-regtest-check.py`](scripts/debitmydata-regtest-check.py) makes a private test
chain with the DebitMyData node (every address type, payments, a whole reward cycle), lets bitviewd
read it, and compares it with the node: every address's balance and coins, the coins in existence,
each block's reward, the addresses, the price and the market cap.

```
scripts/debitmydata-regtest-check.py --node-bin <folder with debitmydatad and debitmydata-cli> \
    --bitviewd target/debug/bitviewd
```
