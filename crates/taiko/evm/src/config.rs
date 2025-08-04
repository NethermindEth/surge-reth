use reth_chainspec::{ChainSpec, Head};
use reth_taiko_forks::TaikoHardfork;

pub(crate) fn revm_spec(chain_spec: &ChainSpec, block: &Head) -> revm_primitives::SpecId {
    if chain_spec.fork(TaikoHardfork::Pacaya).active_at_head(block) {
        revm_primitives::PACAYA
    } else if chain_spec.fork(TaikoHardfork::Ontake).active_at_head(block) {
        revm_primitives::ONTAKE
    } else if chain_spec.fork(TaikoHardfork::Hekla).active_at_head(block) {
        revm_primitives::HEKLA
    } else {
        panic!(
            "invalid hardfork chainspec: expected at least one hardfork, got {:?}",
            chain_spec.hardforks
        )
    }
}
