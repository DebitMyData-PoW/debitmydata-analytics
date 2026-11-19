---
name: debitmydata-analytics
description: "Fetches live and historical DebitMyData (DMD) chain data from DebitMyData Analytics, read from a full node: on-chain metrics (realized price, holder cohorts, supply by age), blocks, transactions, addresses, mempool and fees, mining, hashrate and difficulty, and DMD's reference price in US dollars, computed from the chain since its first block in February 2026. Use when a question needs DebitMyData on-chain, price, mempool, fee, mining or holder-cohort data."
---

# DebitMyData Analytics

DebitMyData (DMD) chain data API at https://analytics.debitmydata.io, read from a full node. Free and read-only, with no API key. Plain HTTPS GETs returning JSON. Help and source: https://github.com/DebitMyData-PoW/debitmydata-analytics.

## Series

17239 time series, each read at an index: `GET /api/series/{series}/{index}` returns the values in `data` and their unit in `type`. `start`: Inclusive start: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end. `end`: Exclusive end: integer index, date (YYYY-MM-DD), or timestamp (ISO 8601). Negative integers count from end.

Ids are snake_case metric names: request a likely one directly, as an unknown id returns 404 with the nearest ids and a wrong index 400 with the valid ones. Otherwise:

1. `GET /api/series/search?q=<q>`: search by meaning with plain words; it returns series ids.
2. `GET /api/series/{series}`: read a returned id's description, supported indexes, value type, unit (what the value type measures), accumulation (how its values combine over time), and the indexes whose values can be null (missing or undefined).
3. `GET /api/series/{series}/{index}`, `GET /api/series/{series}/{index}/latest`, `GET /api/series/bulk?series=<series>&index=<index>`: pick one of those indexes and fetch its values (`start` inclusive, `end` exclusive), the latest value alone, or several comma-separated series at once.

Never invent a series id: only ids the API returned exist. `references/series.md` maps the series by topic, for browsing.

Fetch and compute in one command, Python reading the JSON:

```sh
python3 -c 'import json, urllib.request as r; q = r.Request("https://analytics.debitmydata.io/api/series/bulk?series=<a>,<b>&index=day1&start=2026-03-01&end=2026-03-03", headers={"User-Agent": "debitmydata-analytics-skill"}); a, b = (s["data"] for s in json.load(r.urlopen(q))); print(a, b, b[-1] / a[-1])'
```

Indexes:

- Time buckets, UTC, `start`/`end` as dates or timestamps: `minute10` (10 minutes), `minute30` (30 minutes), `hour1` (1 hour), `hour4` (4 hours), `hour12` (12 hours), `day1` (1 day), `day3` (3 days), `week1` (1 week), `month1` (1 month), `month3` (3 months), `month6` (6 months), `year1` (1 year), `year10` (10 years).
- Positions (block height, epoch, transaction, input, output, or address of a type), `start`/`end` as integers: `halving_epoch`, `difficulty_epoch`, `height`, `tx_index`, `txin_index`, `txout_index`, `empty_script_output_index`, `op_return_output_index`, `p2a_address_index`, `p2ms_output_index`, `p2pk33_address_index`, `p2pk65_address_index`, `p2pkh_address_index`, `p2sh_address_index`, `p2tr_address_index`, `p2wpkh_address_index`, `p2wsh_address_index`, `unknown_script_output_index`.

Traps:

- `end` is exclusive: one block is start 900000, end 900001; one day is start 2025-01-01, end 2025-01-02.
- A `Ratio` or `Multiple` is a plain number (0.55, not 55%); a `Percent` is already a percent (55 means 55%).
- `null` means missing or undefined (before the data exists, or a period with no block), never zero.
- `price` reads 0, not null, before block 74,641 (August 2010): there was no price yet.
- Index names are exact (`day1`, not `day`); a wrong one is rejected with the valid list.
- The latest value is the newest one only, never a past one.
- Convert values only as asked.
- Compute derived numbers at full precision; round only the final answer, if at all.
- Answer in as few commands as you can: fetch and compute in one `python3 -c` as above, with `time.sleep(1)` between requests.

## Other endpoints

- `/health`
- `/version`
- `/api/server/disk`, `/api/server/logs`, `/api/server/sync`
- `/api/series[/list|/count|/indexes|/{series}/{index}/len|/{series}/{index}/data|/{series}/{index}/version]`
- `/api/urpd[/{cohort}|/{cohort}/{point}]`
- `/api/v1/difficulty-adjustment`
- `/api/v1/prices`
- `/api/v1/historical-price`
- `/api/address/{address}[/txs|/utxo|/txs/chain|/txs/mempool|/txs/chain/{after_txid}]`, `/api/address/hash-prefix/{address_type}/{prefix}`
- `/api/v1/validate-address/{address}`
- `/api/block/{hash}[/raw|/txs|/txids|/header|/status|/txid/{index}|/txs/{start_index}]`, `/api/v1/block/{hash}`
- `/api/block-height/{height}`
- `/api/v1/mining/pools[/{time_period}]`, `/api/v1/mining/hashrate[/pools|/{time_period}|/pools/{time_period}]`, `/api/v1/mining/pool/{slug}[/blocks|/hashrate|/blocks/{height}]`, `/api/v1/mining/difficulty-adjustments[/{time_period}]`, `/api/v1/mining/blocks/fees/{time_period}`, `/api/v1/mining/reward-stats/{block_count}`, `/api/v1/mining/blocks/rewards/{time_period}`, `/api/v1/mining/blocks/timestamp/{timestamp}`, `/api/v1/mining/blocks/fee-rates/{time_period}[/precise]`, `/api/v1/mining/blocks/sizes-weights/{time_period}`
- `/api/blocks[/tip/hash|/{height}|/tip/height]`, `/api/v1/blocks[/{height}]`
- `/api/v1/fees/precise`, `/api/v1/fees/recommended`, `/api/v1/fees/mempool-blocks`
- `/api/mempool[/txids|/recent]`
- `/api/v1/replacements`
- `/api/v1/fullrbf/replacements`
- `/api/mempool-blocks/{index}[/diff/{since}]`
- `/api/block-template`
- `/api/tx-index/{index}`
- `/api/v1/cpfp/{txid}`
- `/api/tx/{txid}[/hex|/raw|/status|/outspends|/merkle-proof|/outspend/{vout}|/merkleblock-proof]`, `/api/v1/tx/{txid}/rbf`
- `/api/v1/transaction-times`
- `/api/utxo-set[/{point}|/{point}/diff]`
- `/blindbit/mainnet/info`, `/blindbit/mainnet/block-height`, `/blindbit/mainnet/utxos/{height}`, `/blindbit/mainnet/tweaks/{height}`, `/blindbit/mainnet/block-hash/{height}`, `/blindbit/mainnet/spent-index/{height}`, `/blindbit/mainnet/tweak-index/{height}`, `/blindbit/mainnet/filter/{kind}/{height}`
- `/api/price`
- `/openapi.json`
- `/api.json`

What each returns and takes: `references/endpoints.md`. Files under `references/` are also at https://analytics.debitmydata.io/.well-known/agent-skills/debitmydata-analytics/references/. The full reference with response schemas: https://analytics.debitmydata.io/llms-full.txt.
