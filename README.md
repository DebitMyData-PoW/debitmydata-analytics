# DebitMyData Analytics

On-chain analytics for the DebitMyData (DMD) chain, with an open API, live at
[analytics.debitmydata.io](https://analytics.debitmydata.io). Every block, transaction and address
since the first block becomes thousands of series and charts that anyone can explore, compare and
download, without an account.

It is a fork of [Bitview](https://github.com/bitcoinresearchkit/mono), the Bitcoin Research Kit,
adapted to the DebitMyData chain. Bitview's own documentation is in [docs/](docs/README.md).

## What it offers

| App | |
| :-- | :-- |
| **Studio** | Thousands of ready-made charts of the chain: supply, holders, fees, mining, valuations |
| **Oracle** | The network's hashrate, reference price and difficulty on one screen, from 15 minutes to the whole chain |
| **Explorer** | Every block, transaction and address |
| **Compass** | Where DMD stands against what its holders paid |
| **Globe** | Where the blocks are mined, by each pool's country |
| **DebitMyData Trail** | Every DMD by the price it last moved at, drawn as a terrain |
| **Stack** | A plan buying DMD on a schedule, against the chain's own history |
| **Calendar** | Each day's blocks and pools |
| **Miners** | Which pool first did each new thing on the chain |
| **Catalog** | Every series, browsed like files |
| **API** | Every series, block and address as data (`/api`), for any website or app |

## What differs from Bitview

- **The chain.** DebitMyData's network, its addresses (`D…`, `dm1…`), its block reward (35 DMD,
  halving every 300,000 blocks, starting again after 32 halvings) and its 2.5-minute blocks, 576 a
  day. All in one place: [`crates/brk_types/src/chain.rs`](crates/brk_types/src/chain.rs).
- **The price.** DMD has no market price yet. Bitview reads Bitcoin's price from round-dollar
  payments on its chain; here every block is priced with **DebitMyData's reference price**, the
  DebitMyData Agent Gateway's: the cost of the electricity it takes to mine one DMD (the chain's
  work over the last 4,032 blocks, at 20 J/TH and $0.06 per kWh, divided by the block reward),
  never below $0.001. It is worked out from the chain alone, for every block since the first
  ([`crates/bitview_plugin_price/src/reference.rs`](crates/bitview_plugin_price/src/reference.rs)),
  and shown everywhere as the reference price, not a market price.
- **Mining.** DebitMyData's mining pools instead of Bitcoin's: the public list in
  [DebitMyData-PoW/mining-pools](https://github.com/DebitMyData-PoW/mining-pools) (each pool's name,
  the text it writes in its blocks, its addresses and its country) and the logos in
  [DebitMyData-PoW/mining-pool-logos](https://github.com/DebitMyData-PoW/mining-pool-logos), read
  every day. A new pool appears on every page as soon as it is on the list. The difficulty adjusts
  at every block (LWMA); Bitview's 2,016-block periods are kept for the charts.
- **Bedrock** (the price floors Stack buys at) reads DebitMyData's own history of holders' losses,
  once the chain has 52,560 blocks behind it (about three months).
- **The Oracle** shows the network instead of a payments-based price: hashrate (the default),
  reference price or difficulty, over 15 minutes to the whole chain. Its glowing grid is drawn as
  Bitview's, each block read over 19 spans of time, brightest where most readings agree.
- **The Explorer** draws DebitMyData's small blocks full at 2,000 weight units, so that a block
  never looks empty, and shows the coinbase the next block will hold.
- **The website** speaks of DMD everywhere, in DebitMyData's name and colors, with amounts in DMD
  and data (10^-8 DMD) and fee rates in data/vB. The API keeps Bitview's names (`btc`, `sats`,
  `cents`: DMD, data, millionths of a dollar), so that Bitview's clients and charts work unchanged.
- **Left out**: Bitview's Rarity Meter (calibrated on Bitcoin's history), its Ledger page, the
  Miners page's BIP-110 table (a Bitcoin proposal), its benchmarks, its MCP server, its assistants'
  plugin, and its terms, privacy and support pages.

## Checked against a real node

[`scripts/debitmydata-regtest-check.py`](scripts/debitmydata-regtest-check.py) makes a private
test chain with the DebitMyData node (every address type, payments, a whole reward cycle), lets
the analytics read it, and compares every result with the node: every address's balance and coins,
the coins in existence, each block's reward, the price and the market cap.

## Security

- **No keys, no coins**: the server's node has no wallet, and its RPC answers that server only.
  The analytics read the node's block files with read-only access.
- **The node's IP address stays hidden**: it accepts no connections and reaches its peers through
  Tor, which is kept up to date and checked every 30 minutes.
- **Only Cloudflare reaches the web ports**, and nginx limits each visitor's requests.
- Pool logos from GitHub are checked before use: only plain pictures are kept (no script, no link
  out).
- Nothing secret is in this repository.

## Built with

Rust (bitviewd, from Bitview), a DebitMyData Core node, nginx, Tor; the website in JavaScript.

## Installation

To run it on a server of your own, see **[INSTALL.md](INSTALL.md)**: what it needs, the install,
the commands, updates, the mining pools' sources and the regtest check.

## License

MIT, as Bitview: [LICENSE](LICENSE). Bitview is © its authors (Bitcoin Research Kit & Bitview);
the DebitMyData changes are © DebitMyData.
