//! The DebitMyData (DMD) chain: where it differs from Bitcoin.
//!
//! DebitMyData PoW Core is built from Bitcoin Core, so its blocks and transactions are Bitcoin's,
//! byte for byte. What differs is set here, for its main network and for regtest (a private test
//! chain, used by the tests): the block files' network bytes, the address formats, the block
//! reward and how often it halves, and the block time.
//!
//! The node tells which network it runs (`getblockchaininfo`); [`select`] keeps it for the whole
//! process, before any block is read.

use std::sync::OnceLock;

use bitcoin::{
    PubkeyHash, Script, ScriptBuf, ScriptHash, WitnessProgram, WitnessVersion, base58,
    bech32::{Fe32, Hrp, segwit},
    hashes::Hash,
};

/// A DebitMyData network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chain {
    /// The DebitMyData chain.
    Main,
    /// A private test chain: blocks on demand, the reward halving every 150 blocks.
    Regtest,
}

/// The block reward before its first halving: 35 DMD, in the smallest unit (10^-8 DMD).
pub const INITIAL_SUBSIDY: u64 = 35 * 100_000_000;
/// The block time the difficulty aims at: 2.5 minutes.
pub const TARGET_BLOCK_SECONDS: u64 = 150;
/// Halvings in a reward cycle: after the 32nd, the reward starts again at 35 DMD.
const HALVINGS_PER_CYCLE: u32 = 32;

impl Chain {
    /// From the node's chain name (`getblockchaininfo`'s `chain`).
    pub fn from_core(name: &str) -> Option<Self> {
        match name {
            "main" => Some(Self::Main),
            "regtest" => Some(Self::Regtest),
            _ => None,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Regtest => "regtest",
        }
    }

    /// The bytes before each block in the node's block files (`pchMessageStart`).
    pub const fn magic(self) -> [u8; 4] {
        match self {
            Self::Main => [0xb1, 0xde, 0xf4, 0xc8],
            Self::Regtest => [0xb6, 0xd3, 0x12, 0xb0],
        }
    }

    /// Blocks between two halvings of the reward (`nSubsidyHalvingInterval`).
    pub const fn halving_blocks(self) -> u32 {
        match self {
            Self::Main => 300_000,
            Self::Regtest => 150,
        }
    }

    /// Blocks in a reward cycle (`nSubsidyCycleLength`): the reward starts again at 35 DMD.
    pub const fn subsidy_cycle_blocks(self) -> u32 {
        self.halving_blocks() * HALVINGS_PER_CYCLE
    }

    /// DebitMyData PoW Core's `GetBlockSubsidy`: 35 DMD, halved every
    /// [`halving_blocks`](Self::halving_blocks), starting again at each cycle.
    pub const fn subsidy(self, height: u32) -> u64 {
        let halvings = (height % self.subsidy_cycle_blocks()) / self.halving_blocks();
        INITIAL_SUBSIDY >> halvings
    }

    const fn p2pkh_prefix(self) -> u8 {
        match self {
            Self::Main => 30,
            Self::Regtest => 65,
        }
    }

    const fn p2sh_prefix(self) -> u8 {
        match self {
            Self::Main => 31,
            Self::Regtest => 66,
        }
    }

    /// The prefix of its bech32 addresses: `dm1…` on the main network.
    pub const fn hrp(self) -> &'static str {
        match self {
            Self::Main => "dm",
            Self::Regtest => "dmrt",
        }
    }

    /// The address that pays to `script`, `None` for a script without one (P2PK, multisig,
    /// OP_RETURN, other scripts).
    pub fn address(self, script: &Script) -> Option<String> {
        let bytes = script.as_bytes();
        if script.is_p2pkh() {
            return Some(base58_address(self.p2pkh_prefix(), &bytes[3..23]));
        }
        if script.is_p2sh() {
            return Some(base58_address(self.p2sh_prefix(), &bytes[2..22]));
        }
        let version = script.witness_version()?;
        let hrp = Hrp::parse(self.hrp()).ok()?;
        segwit::encode(hrp, Fe32::try_from(version.to_num()).ok()?, &bytes[2..]).ok()
    }

    /// The script an address of this network pays to. `Err(true)` for an address of another
    /// network (Bitcoin's included), `Err(false)` for text that is not an address.
    pub fn script(self, address: &str) -> Result<ScriptBuf, bool> {
        if let Ok((hrp, version, program)) = segwit::decode(address) {
            if !hrp.as_str().eq_ignore_ascii_case(self.hrp()) {
                return Err(true);
            }
            let version = WitnessVersion::try_from(version).map_err(|_| false)?;
            let program = WitnessProgram::new(version, &program).map_err(|_| false)?;
            return Ok(ScriptBuf::new_witness_program(&program));
        }
        let payload = base58::decode_check(address).map_err(|_| false)?;
        let [prefix, hash @ ..] = payload.as_slice() else {
            return Err(false);
        };
        let hash: [u8; 20] = hash.try_into().map_err(|_| false)?;
        match *prefix {
            p if p == self.p2pkh_prefix() => {
                Ok(ScriptBuf::new_p2pkh(&PubkeyHash::from_byte_array(hash)))
            }
            p if p == self.p2sh_prefix() => {
                Ok(ScriptBuf::new_p2sh(&ScriptHash::from_byte_array(hash)))
            }
            _ => Err(true),
        }
    }
}

fn base58_address(prefix: u8, hash: &[u8]) -> String {
    let mut payload = Vec::with_capacity(21);
    payload.push(prefix);
    payload.extend_from_slice(hash);
    base58::encode_check(&payload)
}

static CURRENT: OnceLock<Chain> = OnceLock::new();

/// Keeps the node's network for this process. `Err` with the one kept when another was already
/// selected: one process reads one chain.
pub fn select(chain: Chain) -> Result<(), Chain> {
    let kept = *CURRENT.get_or_init(|| chain);
    if kept == chain { Ok(()) } else { Err(kept) }
}

/// The network selected, the main network until one is.
pub fn current() -> Chain {
    CURRENT.get().copied().unwrap_or(Chain::Main)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reward_halves_and_starts_again() {
        let main = Chain::Main;
        assert_eq!(main.subsidy(0), 35 * 100_000_000);
        assert_eq!(main.subsidy(299_999), 35 * 100_000_000);
        assert_eq!(main.subsidy(300_000), 1_750_000_000);
        assert_eq!(main.subsidy(9_599_999), (35 * 100_000_000) >> 31);
        assert_eq!(main.subsidy(9_600_000), 35 * 100_000_000);
        assert_eq!(Chain::Regtest.subsidy(150), 1_750_000_000);
        assert_eq!(Chain::Regtest.subsidy(4_800), 35 * 100_000_000);
    }

    #[test]
    fn addresses_both_ways() {
        let hash = [7u8; 20];
        let p2pkh = ScriptBuf::new_p2pkh(&PubkeyHash::from_byte_array(hash));
        let text = Chain::Main.address(&p2pkh).unwrap();
        assert!(text.starts_with('D'), "{text}");
        assert_eq!(Chain::Main.script(&text), Ok(p2pkh.clone()));
        assert_eq!(Chain::Regtest.script(&text), Err(true));

        let p2wpkh = ScriptBuf::new_p2wpkh(&bitcoin::WPubkeyHash::from_byte_array(hash));
        let text = Chain::Main.address(&p2wpkh).unwrap();
        assert!(text.starts_with("dm1q"), "{text}");
        assert_eq!(Chain::Main.script(&text), Ok(p2wpkh.clone()));
        assert_eq!(Chain::Main.script(&text.to_uppercase()), Ok(p2wpkh));

        let p2tr = ScriptBuf::new_witness_program(
            &WitnessProgram::new(WitnessVersion::V1, &[9u8; 32]).unwrap(),
        );
        let text = Chain::Regtest.address(&p2tr).unwrap();
        assert!(text.starts_with("dmrt1p"), "{text}");
        assert_eq!(Chain::Regtest.script(&text), Ok(p2tr));

        // Bitcoin's own addresses are another network's
        assert_eq!(
            Chain::Main.script("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq"),
            Err(true)
        );
        assert_eq!(
            Chain::Main.script("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"),
            Err(true)
        );
        assert_eq!(Chain::Main.script("not an address"), Err(false));

        // the API's examples (addr.rs)
        for example in [
            "DFxKjGyDs1eAyMQsMWxjsKM3SyCNWWZBZF",
            "dm1qw60arpcygrxtce4ng8yj42w6km3a6pgqdvgr80",
        ] {
            let script = Chain::Main.script(example).unwrap();
            assert_eq!(Chain::Main.address(&script).as_deref(), Some(example));
        }
    }
}
