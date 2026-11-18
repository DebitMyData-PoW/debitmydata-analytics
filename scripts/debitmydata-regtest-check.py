#!/usr/bin/env python3
"""Checks bitviewd against a real DebitMyData node, on a private test chain (regtest).

It makes a chain with every address type (D…-style legacy, P2SH-SegWit, dm…1q, dm…1p), payments
and spends between them, and a whole reward cycle (regtest halves the reward every 150 blocks
and starts again at block 4,800), lets bitviewd index it, then compares what bitviewd says with
what the node says: the tip, every address's balance (scantxoutset), the coins in existence
(gettxoutsetinfo), each block's reward (getblockstats), raw blocks and headers (read from the
node's block files), the addresses of the coinbase outputs, and the reference price ($0.001 at
a test chain's difficulty) with the market cap it gives.

    scripts/debitmydata-regtest-check.py --node-bin <folder with debitmydatad, debitmydata-cli> \\
        --bitviewd target/debug/bitviewd [--blocks 5000] [--work /tmp/dmd-regtest-check]

Needs no wallet: the test keys are descriptors the node signs with (descriptorprocesspsbt).
Prints each check and ends with "all checks passed", or stops at the first difference.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import random
import shutil
import socket
import subprocess
import sys
import time
import urllib.request
from decimal import Decimal
from pathlib import Path

DATA_PER_DMD = 100_000_000
MATURITY = 250  # blocks before a coinbase can be spent (DebitMyData PoW Core)
WIF_PREFIX = 193  # regtest private keys
KINDS = ("pkh({})", "sh(wpkh({}))", "wpkh({})", "tr({})")


def free_port() -> int:
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return int(s.getsockname()[1])


def wif(secret: int) -> str:
    raw = bytes([WIF_PREFIX]) + secret.to_bytes(32, "big") + b"\x01"
    raw += hashlib.sha256(hashlib.sha256(raw).digest()).digest()[:4]
    alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
    n, out = int.from_bytes(raw, "big"), ""
    while n:
        n, r = divmod(n, 58)
        out = alphabet[r] + out
    return out


class Node:
    def __init__(self, bin_dir: Path, folder: Path) -> None:
        self.bin, self.folder, self.port = bin_dir, folder, free_port()
        self.base = [f"-datadir={folder}", "-regtest", f"-rpcport={self.port}"]

    def start(self) -> None:
        self.folder.mkdir(parents=True, exist_ok=True)
        # Without a config file the node writes one with the public network's seeds, which a
        # test chain refuses on its next start: an empty one.
        (self.folder / "debitmydata.conf").write_text("")
        subprocess.run(
            [self.bin / "debitmydatad", *self.base, "-daemon", "-listen=0", "-server=1", "-txindex=1"],
            check=True,
            capture_output=True,
        )
        for _ in range(200):
            try:
                self.cli("getblockcount")
                return
            except RuntimeError:
                time.sleep(0.1)
        raise RuntimeError("the node did not start")

    def stop(self) -> None:
        subprocess.run([self.bin / "debitmydata-cli", *self.base, "stop"], capture_output=True)

    def cli(self, *args: object) -> object:
        out = subprocess.run(
            [self.bin / "debitmydata-cli", *self.base, *map(str, args)],
            capture_output=True,
            text=True,
        )
        if out.returncode:
            raise RuntimeError(f"{args[0]}: {out.stderr.strip()}")
        try:
            return json.loads(out.stdout, parse_float=Decimal)
        except json.JSONDecodeError:
            return out.stdout.strip()


def make_chain(node: Node, blocks: int) -> tuple[list[str], list[str]]:
    """Payments between 32 test addresses (8 keys, 4 kinds) until `blocks`."""
    rng = random.Random(7)
    private, public, addresses = [], [], []
    for i in range(1, 9):
        for kind in KINDS:
            descriptor = kind.format(wif(1000 + i))
            info = node.cli("getdescriptorinfo", descriptor)
            private.append(descriptor)
            public.append(info["descriptor"])
            addresses.append(node.cli("deriveaddresses", info["descriptor"])[0])

    def mine(count: int) -> None:
        for start in range(0, count, 250):
            node.cli("generatetoaddress", min(250, count - start), rng.choice(addresses))

    def spendable() -> list[dict]:
        tip = int(node.cli("getblockcount"))
        found = node.cli("scantxoutset", "start", json.dumps(public))["unspents"]
        return [u for u in found if not u["coinbase"] or tip - u["height"] >= MATURITY]

    mine(MATURITY + 50)
    while int(node.cli("getblockcount")) < blocks:
        coins = spendable()
        rng.shuffle(coins)
        for _ in range(rng.randint(2, 10)):
            take = [coins.pop() for _ in range(min(len(coins), rng.randint(1, 3)))]
            if not take:
                break
            left = sum(Decimal(c["amount"]) for c in take) - Decimal("0.0002") * len(take)
            outputs: dict[str, Decimal] = {}
            for destination in rng.sample(addresses, rng.randint(1, 4)):
                part = (left * Decimal(rng.uniform(0.05, 0.6))).quantize(Decimal("0.00000001"))
                if part > Decimal("0.001") and destination not in outputs:
                    outputs[destination] = part
                    left -= part
            if left <= Decimal("0.001") or not outputs:
                continue
            change = rng.choice(addresses)
            outputs[change] = outputs.get(change, Decimal(0)) + left
            psbt = node.cli(
                "createpsbt",
                json.dumps([{"txid": c["txid"], "vout": c["vout"]} for c in take]),
                json.dumps({a: str(v) for a, v in outputs.items()}),
            )
            signed = node.cli("descriptorprocesspsbt", psbt, json.dumps(private))
            final = node.cli("finalizepsbt", signed["psbt"])
            node.cli("sendrawtransaction", final["hex"])
        mine(min(rng.randint(1, 40), blocks - int(node.cli("getblockcount"))))
    return addresses, public


def get(port: int, path: str) -> object:
    with urllib.request.urlopen(f"http://127.0.0.1:{port}{path}", timeout=60) as response:
        body = response.read()
    try:
        return json.loads(body)
    except json.JSONDecodeError:
        return body.decode()


def same(what: str, ours: object, node: object) -> None:
    if ours != node:
        sys.exit(f"DIFFERENT {what}: bitviewd {ours!r}, node {node!r}")
    print(f"ok  {what}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--node-bin", type=Path, required=True)
    parser.add_argument("--bitviewd", type=Path, required=True)
    parser.add_argument("--blocks", type=int, default=5000)
    parser.add_argument("--work", type=Path, default=Path("/tmp/dmd-regtest-check"))
    parser.add_argument("--keep", action="store_true", help="reuse a chain made before")
    args = parser.parse_args()

    chain_file = args.work / "chain.json"
    node = Node(args.node_bin, args.work / "node")
    if not (args.keep and chain_file.exists()):
        shutil.rmtree(args.work, ignore_errors=True)
        node.start()
        addresses, descriptors = make_chain(node, args.blocks)
        chain_file.write_text(json.dumps({"addresses": addresses, "descriptors": descriptors}))
    else:
        node.start()
        saved = json.loads(chain_file.read_text())
        addresses, descriptors = saved["addresses"], saved["descriptors"]
    tip = int(node.cli("getblockcount"))
    print(f"chain: {tip} blocks")

    port = free_port()
    data = args.work / "bitview"
    shutil.rmtree(data, ignore_errors=True)
    data.mkdir(parents=True)
    log = (args.work / "bitviewd.log").open("w")
    server = subprocess.Popen(
        [
            str(args.bitviewd),
            "--bitviewdir", str(data),
            "--bitcoindir", str(node.folder / "regtest"),
            "--rpcport", str(node.port),
            "--serverport", str(port),
        ],
        stdout=log,
        stderr=subprocess.STDOUT,
    )
    try:
        started = time.monotonic()
        while True:
            if server.poll() is not None:
                sys.exit(f"bitviewd stopped (exit {server.returncode}): see {log.name}")
            try:
                if int(get(port, "/api/blocks/tip/height")) == tip:
                    break
            except Exception:
                pass
            if time.monotonic() - started > 3600:
                sys.exit("bitviewd did not reach the tip within an hour")
            time.sleep(2)
        print(f"bitviewd synced in {time.monotonic() - started:.0f} s")
        check(port, node, tip, addresses, descriptors)
    finally:
        server.terminate()
        server.wait(timeout=60)
        node.stop()
    print("all checks passed")


def check(port: int, node: Node, tip: int, addresses: list[str], descriptors: list[str]) -> None:
    same("tip hash", get(port, "/api/blocks/tip/hash"), node.cli("getbestblockhash"))

    # the coins in existence: the node's UTXO set (the genesis reward is in no UTXO set)
    utxo_set, info = get(port, "/api/utxo-set"), node.cli("gettxoutsetinfo")
    same(
        "coins in existence",
        round(Decimal(str(utxo_set["supply"])) * DATA_PER_DMD),
        round(Decimal(info["total_amount"]) * DATA_PER_DMD),
    )
    same("unspent outputs", utxo_set["count"], info["txouts"])
    same(
        "circulating supply series",
        Decimal(str(get(port, "/api/series/circulating_supply/height/latest"))),
        Decimal(info["total_amount"]),
    )

    # the reference price: a test chain's difficulty is tiny, so its mining cost is far below
    # the minimum rate, $0.001 per DMD, at every block (the formula's own test: reference.rs)
    same("price now", get(port, "/api/price"), 0.001)
    prices = get(port, f"/api/series/price/height?start=0&end={tip + 1}")["data"]
    same("price of every block", set(prices), {0.001})
    same(
        "market cap",
        round(Decimal(str(get(port, "/api/series/market_cap/height/latest"))), 6),
        round(Decimal(info["total_amount"]) * Decimal("0.001"), 6),
    )

    # every address's balance and coins, from the node's own coins
    for address, descriptor in zip(addresses, descriptors, strict=True):
        found = node.cli("scantxoutset", "start", json.dumps([descriptor]))
        stats = get(port, f"/api/address/{address}")["chain_stats"]
        ours = stats["funded_txo_sum"] - stats["spent_txo_sum"]
        same(f"balance of {address}", ours, round(Decimal(found["total_amount"]) * DATA_PER_DMD))
        utxos = get(port, f"/api/address/{address}/utxo")
        same(f"coins of {address}", len(utxos), len(found["unspents"]))

    # each block's reward: 35 DMD, halved every 150 blocks, again 35 DMD from block 4,800
    for height in sorted({1, 149, 150, 151, 300, 4_799, 4_800, 4_801, tip} & set(range(tip + 1))):
        block = get(port, f"/api/v1/blocks/{height}")[0]
        stats = node.cli("getblockstats", height, json.dumps(["subsidy", "totalfee"]))
        same(
            f"reward of block {height}",
            block["extras"]["reward"],
            stats["subsidy"] + stats["totalfee"],
        )

    # a block's raw bytes and header, as the node has them (read from its block files)
    for height in (0, tip):
        block_hash = node.cli("getblockhash", height)
        with urllib.request.urlopen(
            f"http://127.0.0.1:{port}/api/block/{block_hash}/raw", timeout=60
        ) as response:
            same(f"raw block {height}", response.read().hex(), node.cli("getblock", block_hash, 0))
        same(
            f"header of block {height}",
            get(port, f"/api/block/{block_hash}/header"),
            node.cli("getblockheader", block_hash, "false"),
        )

    # the coinbase output's address, in DebitMyData's format
    for height in (300, tip):
        block_hash = node.cli("getblockhash", height)
        ours = get(port, f"/api/block/{block_hash}/txs")[0]["vout"][0]
        theirs = node.cli("getblock", block_hash, 2)["tx"][0]["vout"][0]["scriptPubKey"]
        same(
            f"coinbase address of block {height}",
            ours.get("scriptpubkey_address"),
            theirs.get("address"),
        )


if __name__ == "__main__":
    main()
