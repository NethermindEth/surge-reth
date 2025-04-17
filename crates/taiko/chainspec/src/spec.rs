use std::sync::{Arc, LazyLock};

use reth_chainspec::ChainSpec;
use reth_taiko_forks::{
    hardfork::{TAIKO_A7_HARDFORKS, TAIKO_DEV_HARDFORKS, TAIKO_MAINNET_HARDFORKS},
    CHAIN_HEKLA_TESTNET, CHAIN_INTERNAL_TESTNET, CHAIN_MAINNET,
};

use crate::TaikoChainSpec;

/// The Taiko A7 spec
pub static TAIKO_A7: LazyLock<Arc<TaikoChainSpec>> = LazyLock::new(|| {
    let genesis = serde_json::from_str(include_str!("../res/genesis/hekla.json"))
        .expect("Can't deserialize Unichain genesis json");
    let hardforks = TAIKO_A7_HARDFORKS.clone();
    TaikoChainSpec {
        inner: ChainSpec {
            chain: CHAIN_HEKLA_TESTNET,
            genesis,
            genesis_hash: Default::default(), /* TODO: This field will be filled in later, but
                                               * better
                                               * to create constant for it */
            paris_block_and_final_difficulty: None,
            hardforks,
            deposit_contract: None,
            ..Default::default()
        },
    }
    .into()
});

/// The Taiko devnet spec
pub static TAIKO_DEV: LazyLock<Arc<TaikoChainSpec>> = LazyLock::new(|| {
    let hardforks = TAIKO_DEV_HARDFORKS.clone();
    TaikoChainSpec {
        inner: ChainSpec {
            chain: CHAIN_INTERNAL_TESTNET,
            genesis_hash: Default::default(), /* TODO: This field will be filled in later, but
                                               * better
                                               * to create constant for it */
            paris_block_and_final_difficulty: None,
            hardforks,
            deposit_contract: None,
            ..Default::default()
        },
    }
    .into()
});

/// The Taiko Mainnet spec
pub static TAIKO_MAINNET: LazyLock<Arc<TaikoChainSpec>> = LazyLock::new(|| {
    let genesis = serde_json::from_str(include_str!("../res/genesis/mainnet.json"))
        .expect("Can't deserialize Unichain genesis json");
    let hardforks = TAIKO_MAINNET_HARDFORKS.clone();
    TaikoChainSpec {
        inner: ChainSpec {
            chain: CHAIN_MAINNET,
            genesis,
            genesis_hash: Default::default(), /* TODO: This field will be filled in later, but
                                               * better
                                               * to create constant for it */
            paris_block_and_final_difficulty: None,
            hardforks,
            deposit_contract: None,
            ..Default::default()
        },
    }
    .into()
});

#[cfg(test)]
mod tests {

    use reth_chainspec::{test_fork_ids, EthereumHardfork, Hardfork};
    use reth_ethereum_forks::{ForkHash, ForkId, Head};
    use reth_taiko_forks::TaikoHardfork;

    use super::*;

    fn test_hardfork_fork_ids(spec: &ChainSpec, cases: &[(Box<dyn Hardfork>, ForkId)]) {
        for (hardfork, expected_id) in cases {
            if let Some(computed_id) = spec.hardfork_fork_id(hardfork.clone()) {
                assert_eq!(
                    expected_id, &computed_id,
                    "Expected fork ID {expected_id:?}, computed fork ID {computed_id:?} for hardfork {}", hardfork.name()
                );
                if hardfork.name() == EthereumHardfork::Shanghai.name() {
                    // if matches!(hardfork, EthereumHardfork::Shanghai) {
                    if let Some(shangai_id) = spec.shanghai_fork_id() {
                        assert_eq!(
                            expected_id, &shangai_id,
                            "Expected fork ID {expected_id:?}, computed fork ID {computed_id:?} for Shanghai hardfork"
                        );
                    } else {
                        panic!("Expected ForkCondition to return Some for Hardfork::Shanghai");
                    }
                }
            }
        }
    }

    #[test]
    fn taiko_a7_hardfork_fork_ids() {
        test_hardfork_fork_ids(
            &TAIKO_A7,
            &[
                (
                    EthereumHardfork::Frontier.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 840512 },
                ),
                (
                    EthereumHardfork::Homestead.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 840512 },
                ),
                (
                    EthereumHardfork::Dao.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 840512 },
                ),
                (
                    EthereumHardfork::Tangerine.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 840512 },
                ),
                (
                    EthereumHardfork::SpuriousDragon.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 840512 },
                ),
                (
                    EthereumHardfork::Byzantium.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 840512 },
                ),
                (
                    EthereumHardfork::Constantinople.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 840512 },
                ),
                (
                    EthereumHardfork::Petersburg.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 840512 },
                ),
                (
                    EthereumHardfork::Istanbul.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 840512 },
                ),
                (
                    EthereumHardfork::Berlin.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 840512 },
                ),
                (
                    EthereumHardfork::London.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 840512 },
                ),
                (
                    EthereumHardfork::Paris.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 840512 },
                ),
                (
                    EthereumHardfork::Shanghai.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 840512 },
                ),
                (
                    TaikoHardfork::Hekla.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 840512 },
                ),
                (
                    TaikoHardfork::Ontake.boxed(),
                    ForkId { hash: ForkHash([0xbd, 0x19, 0x32, 0x5c]), next: 0 },
                ),
            ],
        );
    }

    #[test]
    fn taiko_dev_hardfork_fork_ids() {
        test_hardfork_fork_ids(
            &TAIKO_DEV,
            &[
                (
                    EthereumHardfork::Frontier.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 2000 },
                ),
                (
                    EthereumHardfork::Homestead.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 2000 },
                ),
                (
                    EthereumHardfork::Dao.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 2000 },
                ),
                (
                    EthereumHardfork::Tangerine.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 2000 },
                ),
                (
                    EthereumHardfork::SpuriousDragon.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 2000 },
                ),
                (
                    EthereumHardfork::Byzantium.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 2000 },
                ),
                (
                    EthereumHardfork::Constantinople.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 2000 },
                ),
                (
                    EthereumHardfork::Petersburg.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 2000 },
                ),
                (
                    EthereumHardfork::Istanbul.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 2000 },
                ),
                (
                    EthereumHardfork::Berlin.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 2000 },
                ),
                (
                    EthereumHardfork::London.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 2000 },
                ),
                (
                    EthereumHardfork::Paris.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 2000 },
                ),
                (
                    EthereumHardfork::Shanghai.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 2000 },
                ),
                (
                    TaikoHardfork::Hekla.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 2000 },
                ),
                (
                    TaikoHardfork::Ontake.boxed(),
                    ForkId { hash: ForkHash([0xa1, 0x58, 0x58, 0x67]), next: 0 },
                ),
            ],
        );
    }

    #[test]
    fn taiko_mainnet_hardfork_fork_ids() {
        test_hardfork_fork_ids(
            &TAIKO_MAINNET,
            &[
                (
                    EthereumHardfork::Frontier.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 538304 },
                ),
                (
                    EthereumHardfork::Homestead.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 538304 },
                ),
                (
                    EthereumHardfork::Dao.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 538304 },
                ),
                (
                    EthereumHardfork::Tangerine.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 538304 },
                ),
                (
                    EthereumHardfork::SpuriousDragon.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 538304 },
                ),
                (
                    EthereumHardfork::Byzantium.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 538304 },
                ),
                (
                    EthereumHardfork::Constantinople.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 538304 },
                ),
                (
                    EthereumHardfork::Petersburg.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 538304 },
                ),
                (
                    EthereumHardfork::Istanbul.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 538304 },
                ),
                (
                    EthereumHardfork::Berlin.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 538304 },
                ),
                (
                    EthereumHardfork::London.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 538304 },
                ),
                (
                    EthereumHardfork::Paris.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 538304 },
                ),
                (
                    EthereumHardfork::Shanghai.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 538304 },
                ),
                (
                    TaikoHardfork::Hekla.boxed(),
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 538304 },
                ),
                (
                    TaikoHardfork::Ontake.boxed(),
                    ForkId { hash: ForkHash([0x74, 0xa1, 0x1e, 0x09]), next: 0 },
                ),
            ],
        );
    }

    #[test]
    fn taiko_a7_fork_ids() {
        test_fork_ids(
            &TAIKO_A7,
            &[
                (
                    Head { number: 0, ..Default::default() },
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 840512 },
                ),
                (
                    Head { number: 840512, ..Default::default() },
                    ForkId { hash: ForkHash([0xbd, 0x19, 0x32, 0x5c]), next: 0 },
                ),
            ],
        );
    }

    #[test]
    fn taiko_dev_fork_ids() {
        test_fork_ids(
            &TAIKO_DEV,
            &[
                (
                    Head { number: 0, ..Default::default() },
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 2000 },
                ),
                (
                    Head { number: 2000, ..Default::default() },
                    ForkId { hash: ForkHash([0xa1, 0x58, 0x58, 0x67]), next: 0 },
                ),
            ],
        );
    }

    #[test]
    fn taiko_mainnet_fork_ids() {
        test_fork_ids(
            &TAIKO_MAINNET,
            &[
                (
                    Head { number: 0, ..Default::default() },
                    ForkId { hash: ForkHash([0xfc, 0xfd, 0x4b, 0xce]), next: 538304 },
                ),
                (
                    Head { number: 538304, ..Default::default() },
                    ForkId { hash: ForkHash([0x74, 0xa1, 0x1e, 0x09]), next: 0 },
                ),
            ],
        );
    }
}
