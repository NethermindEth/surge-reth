//! Hard forks of optimism protocol.

use alloc::{boxed::Box, format, string::String};
use alloy_primitives::U256;
use core::{
    any::Any,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use alloy_chains::Chain;
use reth_ethereum_forks::{hardfork, EthereumHardfork, ForkCondition, Hardfork};

/// The chain for the Taiko mainnet.
pub const CHAIN_MAINNET: Chain = Chain::taiko();
/// The chain for the Taiko internal testnet.
pub const CHAIN_INTERNAL_TESTNET: Chain = Chain::from_id_unchecked(167001);
/// The chain for the Taiko katla testnet.
pub const CHAIN_KATLA_TESTNET: Chain = Chain::from_id_unchecked(167008);
/// The chain for the Taiko hekla testnet.
pub const CHAIN_HEKLA_TESTNET: Chain = Chain::taiko_hekla();
/// The chain for the Taiko preconf devnet.
pub const CHAIN_PERCONF_DEVNET: Chain = Chain::from_id_unchecked(167010);

hardfork!(
    /// The name of an taiko hardfork.
    ///
    /// When building a list of hardforks for a chain, it's still expected to mix with
    /// [`TaikoHardfork`].
    TaikoHardfork {
        /// The Kalta hardfork.
        Kalta,
        /// The Hekla hardfork.
        Hekla,
        /// The Ontake hardfork.
        Ontake,
        /// The Pacaya hardfork.
        Pacaya,
    }
);

impl TaikoHardfork {
    /// Taiko A7 list of hardforks.
    pub fn taiko_a7() -> [(Box<dyn Hardfork>, ForkCondition); 15] {
        [
            (EthereumHardfork::Frontier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Homestead.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Dao.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Tangerine.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::SpuriousDragon.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Byzantium.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Constantinople.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Petersburg.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Istanbul.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Berlin.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::London.boxed(), ForkCondition::Block(0)),
            (
                EthereumHardfork::Paris.boxed(),
                ForkCondition::TTD { fork_block: None, total_difficulty: U256::ZERO },
            ),
            (EthereumHardfork::Shanghai.boxed(), ForkCondition::Timestamp(0)),
            (Self::Hekla.boxed(), ForkCondition::Block(0)),
            (
                Self::Ontake.boxed(),
                ForkCondition::Block(
                    std::env::var("HEKLA_ONTAKE_HEIGHT")
                        .map_or(840512, |h| h.parse().unwrap_or(840512)),
                ),
            ),
        ]
    }

    /// Taiko DEV list of hardforks.
    pub fn taiko_dev() -> [(Box<dyn Hardfork>, ForkCondition); 15] {
        [
            (EthereumHardfork::Frontier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Homestead.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Dao.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Tangerine.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::SpuriousDragon.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Byzantium.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Constantinople.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Petersburg.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Istanbul.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Berlin.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::London.boxed(), ForkCondition::Block(0)),
            (
                EthereumHardfork::Paris.boxed(),
                ForkCondition::TTD { fork_block: None, total_difficulty: U256::from(0) },
            ),
            (EthereumHardfork::Shanghai.boxed(), ForkCondition::Timestamp(0)),
            (Self::Hekla.boxed(), ForkCondition::Block(0)),
            (
                Self::Ontake.boxed(),
                ForkCondition::Block(
                    std::env::var("DEV_ONTAKE_HEIGHT").map_or(2000, |h| h.parse().unwrap_or(2000)),
                ),
            ),
        ]
    }

    /// Taiko mainnet list of hardforks.
    pub fn taiko_mainnet() -> [(Box<dyn Hardfork>, ForkCondition); 15] {
        [
            (EthereumHardfork::Frontier.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Homestead.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Dao.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Tangerine.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::SpuriousDragon.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Byzantium.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Constantinople.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Petersburg.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Istanbul.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::Berlin.boxed(), ForkCondition::Block(0)),
            (EthereumHardfork::London.boxed(), ForkCondition::Block(0)),
            (
                EthereumHardfork::Paris.boxed(),
                ForkCondition::TTD { fork_block: None, total_difficulty: U256::from(0) },
            ),
            (EthereumHardfork::Shanghai.boxed(), ForkCondition::Timestamp(0)),
            (Self::Hekla.boxed(), ForkCondition::Block(0)),
            (
                Self::Ontake.boxed(),
                ForkCondition::Block(
                    std::env::var("MAINNET_ONTAKE_HEIGHT")
                        .map_or(538304, |h| h.parse().unwrap_or(538304)),
                ),
            ),
        ]
    }

    /// Retrieves the activation block for the specified hardfork on the given chain.
    pub fn activation_block<H: Hardfork>(self, fork: H, chain: Chain) -> Option<u64> {
        match chain {
            CHAIN_MAINNET => Self::base_mainnet_activation_block(fork),
            CHAIN_INTERNAL_TESTNET => Self::base_internal_activation_block(fork),
            CHAIN_KATLA_TESTNET => Self::base_katla_activation_block(fork),
            CHAIN_HEKLA_TESTNET => Self::base_hekla_activation_block(fork),
            _ => None,
        }
    }

    /// Retrieves the activation timestamp for the specified hardfork on the given chain.
    pub fn activation_timestamp<H: Hardfork>(self, fork: H, chain: Chain) -> Option<u64> {
        match chain {
            CHAIN_MAINNET => Self::base_mainnet_activation_timestamp(fork),
            CHAIN_INTERNAL_TESTNET => Self::base_internal_activation_timestamp(fork),
            CHAIN_KATLA_TESTNET => Self::base_kalta_activation_timestamp(fork),
            CHAIN_HEKLA_TESTNET => Self::base_hekla_activation_timestamp(fork),
            _ => None,
        }
    }

    /// Retrieves the activation block for the specified hardfork on the Base Internal testnet.
    pub fn base_internal_activation_block<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Frontier |
                EthereumHardfork::Homestead |
                EthereumHardfork::Tangerine |
                EthereumHardfork::SpuriousDragon |
                EthereumHardfork::Byzantium |
                EthereumHardfork::Constantinople |
                EthereumHardfork::Petersburg |
                EthereumHardfork::Istanbul |
                EthereumHardfork::Berlin |
                EthereumHardfork::London |
                EthereumHardfork::Paris |
                EthereumHardfork::Shanghai => Some(0),
                _ => None,
            },
            |_fork| Some(0),
        )
    }

    /// Retrieves the activation block for the specified hardfork on the Base Katla testnet.
    pub fn base_katla_activation_block<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Frontier |
                EthereumHardfork::Homestead |
                EthereumHardfork::Tangerine |
                EthereumHardfork::SpuriousDragon |
                EthereumHardfork::Byzantium |
                EthereumHardfork::Constantinople |
                EthereumHardfork::Petersburg |
                EthereumHardfork::Istanbul |
                EthereumHardfork::Berlin |
                EthereumHardfork::London |
                EthereumHardfork::Paris |
                EthereumHardfork::Shanghai => Some(0),
                _ => None,
            },
            |_fork| Some(0),
        )
    }

    /// Retrieves the activation block for the specified hardfork on the Base Hekla testnet.
    pub fn base_hekla_activation_block<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Frontier |
                EthereumHardfork::Homestead |
                EthereumHardfork::Tangerine |
                EthereumHardfork::SpuriousDragon |
                EthereumHardfork::Byzantium |
                EthereumHardfork::Constantinople |
                EthereumHardfork::Petersburg |
                EthereumHardfork::Istanbul |
                EthereumHardfork::Berlin |
                EthereumHardfork::London |
                EthereumHardfork::Paris |
                EthereumHardfork::Shanghai => Some(0),
                _ => None,
            },
            |_fork| Some(0),
        )
    }

    /// Retrieves the activation block for the specified hardfork on the Base mainnet.
    pub fn base_mainnet_activation_block<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Frontier |
                EthereumHardfork::Homestead |
                EthereumHardfork::Tangerine |
                EthereumHardfork::SpuriousDragon |
                EthereumHardfork::Byzantium |
                EthereumHardfork::Constantinople |
                EthereumHardfork::Petersburg |
                EthereumHardfork::Paris |
                EthereumHardfork::Shanghai => Some(0),
                EthereumHardfork::Istanbul => Some(1561651),
                EthereumHardfork::Berlin => Some(4460644),
                EthereumHardfork::London => Some(5062605),
                _ => None,
            },
            |_fork| Some(0),
        )
    }

    /// Retrieves the activation timestamp for the specified hardfork on the Base Internal testnet.
    pub fn base_internal_activation_timestamp<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Frontier |
                EthereumHardfork::Homestead |
                EthereumHardfork::Tangerine |
                EthereumHardfork::SpuriousDragon |
                EthereumHardfork::Byzantium |
                EthereumHardfork::Constantinople |
                EthereumHardfork::Petersburg |
                EthereumHardfork::Istanbul |
                EthereumHardfork::Berlin |
                EthereumHardfork::London |
                EthereumHardfork::Paris |
                EthereumHardfork::Shanghai => Some(0),
                _ => None,
            },
            |_fork| Some(0),
        )
    }

    /// Retrieves the activation timestamp for the specified hardfork on the Base Kalta testnet.
    pub fn base_kalta_activation_timestamp<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Frontier |
                EthereumHardfork::Homestead |
                EthereumHardfork::Tangerine |
                EthereumHardfork::SpuriousDragon |
                EthereumHardfork::Byzantium |
                EthereumHardfork::Constantinople |
                EthereumHardfork::Petersburg |
                EthereumHardfork::Istanbul |
                EthereumHardfork::Berlin |
                EthereumHardfork::London |
                EthereumHardfork::Paris |
                EthereumHardfork::Shanghai => Some(0),
                _ => None,
            },
            |_fork| Some(0),
        )
    }

    /// Retrieves the activation timestamp for the specified hardfork on the Base Hekla testnet.
    pub fn base_hekla_activation_timestamp<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Frontier |
                EthereumHardfork::Homestead |
                EthereumHardfork::Tangerine |
                EthereumHardfork::SpuriousDragon |
                EthereumHardfork::Byzantium |
                EthereumHardfork::Constantinople |
                EthereumHardfork::Petersburg |
                EthereumHardfork::Istanbul |
                EthereumHardfork::Berlin |
                EthereumHardfork::London |
                EthereumHardfork::Paris |
                EthereumHardfork::Shanghai => Some(0),
                _ => None,
            },
            |_fork| Some(0),
        )
    }

    /// Retrieves the activation timestamp for the specified hardfork on the Base Kalta testnet.
    pub fn base_ontake_activation_timestamp<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Frontier |
                EthereumHardfork::Homestead |
                EthereumHardfork::Tangerine |
                EthereumHardfork::SpuriousDragon |
                EthereumHardfork::Byzantium |
                EthereumHardfork::Constantinople |
                EthereumHardfork::Petersburg |
                EthereumHardfork::Istanbul |
                EthereumHardfork::Berlin |
                EthereumHardfork::London |
                EthereumHardfork::Paris |
                EthereumHardfork::Shanghai => Some(0),
                _ => None,
            },
            |_fork| Some(0),
        )
    }

    /// Retrieves the activation timestamp for the specified hardfork on the Base mainnet.
    pub fn base_mainnet_activation_timestamp<H: Hardfork>(fork: H) -> Option<u64> {
        match_hardfork(
            fork,
            |fork| match fork {
                EthereumHardfork::Frontier |
                EthereumHardfork::Homestead |
                EthereumHardfork::Tangerine |
                EthereumHardfork::SpuriousDragon |
                EthereumHardfork::Byzantium |
                EthereumHardfork::Constantinople |
                EthereumHardfork::Petersburg |
                EthereumHardfork::Istanbul |
                EthereumHardfork::Berlin |
                EthereumHardfork::London |
                EthereumHardfork::Paris |
                EthereumHardfork::Shanghai => Some(0),
                _ => None,
            },
            |_fork| Some(0),
        )
    }
}

/// Match helper method since it's not possible to match on `dyn Hardfork`
fn match_hardfork<H, HF, OHF>(fork: H, hardfork_fn: HF, taiko_hardfork_fn: OHF) -> Option<u64>
where
    H: Hardfork,
    HF: Fn(&EthereumHardfork) -> Option<u64>,
    OHF: Fn(&TaikoHardfork) -> Option<u64>,
{
    let fork: &dyn Any = &fork;
    if let Some(fork) = fork.downcast_ref::<EthereumHardfork>() {
        return hardfork_fn(fork);
    }
    fork.downcast_ref::<TaikoHardfork>().and_then(taiko_hardfork_fn)
}
