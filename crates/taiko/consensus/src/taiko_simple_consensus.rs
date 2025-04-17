//! Simple Beacon consensus implementation.

use alloy_consensus::EMPTY_OMMER_ROOT_HASH;
use alloy_primitives::B64;
use reth_chainspec::{EthChainSpec, EthereumHardfork};
use reth_consensus::{
    Consensus, ConsensusError, FullConsensus, HeaderValidator, PostExecutionInput,
};
use reth_consensus_common::validation::{
    validate_4844_header_standalone, validate_against_parent_4844,
    validate_against_parent_hash_number, validate_header_base_fee, validate_header_extradata,
    validate_header_gas,
};
use reth_ethereum_consensus::validate_block_post_execution;
use reth_primitives::{
    Block, BlockBody, BlockWithSenders, EthereumHardforks, Header, NodePrimitives, Receipt,
    SealedBlock, SealedHeader,
};
use revm_primitives::U256;
use std::{fmt::Debug, sync::Arc, time::SystemTime};

/// Taiko beacon consensus
///
/// This consensus engine does basic checks as outlined in the execution specs.
#[derive(Debug)]
pub struct TaikoSimpleBeaconConsensus<ChainSpec> {
    /// Configuration
    chain_spec: Arc<ChainSpec>,
}

impl<ChainSpec> TaikoSimpleBeaconConsensus<ChainSpec>
where
    ChainSpec: Send + Sync + EthChainSpec + EthereumHardforks + Debug,
{
    /// Create a new instance of [`TaikoSimpleBecaonConsensus`]
    pub const fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self { chain_spec }
    }
}

impl<ChainSpec, N> FullConsensus<N> for TaikoSimpleBeaconConsensus<ChainSpec>
where
    ChainSpec: Send + Sync + EthChainSpec + EthereumHardforks + Debug,
    N: NodePrimitives<
        BlockHeader = Header,
        BlockBody = BlockBody,
        Block = Block,
        Receipt = Receipt,
    >,
{
    fn validate_block_post_execution(
        &self,
        block: &BlockWithSenders,
        input: PostExecutionInput<'_>,
    ) -> Result<(), ConsensusError> {
        validate_block_post_execution(block, &self.chain_spec, input.receipts, input.requests)
    }
}

impl<ChainSpec: Send + Sync + EthChainSpec + EthereumHardforks + Debug> HeaderValidator
    for TaikoSimpleBeaconConsensus<ChainSpec>
{
    fn validate_header(&self, header: &SealedHeader) -> Result<(), ConsensusError> {
        validate_header_gas(header.header())?;
        validate_header_base_fee(header.header(), &self.chain_spec)?;

        if !header.is_zero_difficulty() {
            return Err(ConsensusError::TheMergeDifficultyIsNotZero);
        }

        if header.nonce != B64::ZERO {
            return Err(ConsensusError::TheMergeNonceIsNotZero);
        }

        if header.ommers_hash != EMPTY_OMMER_ROOT_HASH {
            return Err(ConsensusError::TheMergeOmmerRootIsNotEmpty);
        }

        // Post-merge, the consensus layer is expected to perform checks such that the block
        // timestamp is a function of the slot. This is different from pre-merge, where blocks
        // are only allowed to be in the future (compared to the system's clock) by a certain
        // threshold.
        //
        // Block validation with respect to the parent should ensure that the block timestamp
        // is greater than its parent timestamp.

        // validate header extradata for all networks post merge
        validate_header_extradata(header.header())?;

        // EIP-4895: Beacon chain push withdrawals as operations
        if self.chain_spec.is_shanghai_active_at_timestamp(header.timestamp) &&
            header.withdrawals_root.is_none()
        {
            return Err(ConsensusError::WithdrawalsRootMissing);
        } else if !self.chain_spec.is_shanghai_active_at_timestamp(header.timestamp) &&
            header.withdrawals_root.is_some()
        {
            return Err(ConsensusError::WithdrawalsRootUnexpected);
        }

        // Ensures that EIP-4844 fields are valid once cancun is active.
        if self.chain_spec.is_cancun_active_at_timestamp(header.timestamp) {
            validate_4844_header_standalone(header.header())?;
        } else if header.blob_gas_used.is_some() {
            return Err(ConsensusError::BlobGasUsedUnexpected);
        } else if header.excess_blob_gas.is_some() {
            return Err(ConsensusError::ExcessBlobGasUnexpected);
        } else if header.parent_beacon_block_root.is_some() {
            return Err(ConsensusError::ParentBeaconBlockRootUnexpected);
        }

        if self.chain_spec.is_prague_active_at_timestamp(header.timestamp) {
            if header.requests_hash.is_none() {
                return Err(ConsensusError::RequestsHashMissing);
            }
        } else if header.requests_hash.is_some() {
            return Err(ConsensusError::RequestsHashUnexpected);
        }

        Ok(())
    }

    fn validate_header_against_parent(
        &self,
        header: &SealedHeader,
        parent: &SealedHeader,
    ) -> Result<(), ConsensusError> {
        validate_against_parent_hash_number(header.header(), parent)?;

        validate_against_parent_timestamp_inclusive(header, parent)?;

        // ensure that the blob gas fields for this block
        if self.chain_spec.is_cancun_active_at_timestamp(header.timestamp) {
            validate_against_parent_4844(header.header(), parent)?;
        }

        Ok(())
    }

    fn validate_header_with_total_difficulty(
        &self,
        header: &Header,
        total_difficulty: U256,
    ) -> Result<(), ConsensusError> {
        let is_post_merge = self
            .chain_spec
            .fork(EthereumHardfork::Paris)
            .active_at_ttd(total_difficulty, header.difficulty);

        if is_post_merge {
            if !header.is_zero_difficulty() {
                return Err(ConsensusError::TheMergeDifficultyIsNotZero)
            }

            if !header.nonce.is_zero() {
                return Err(ConsensusError::TheMergeNonceIsNotZero)
            }

            if header.ommers_hash != EMPTY_OMMER_ROOT_HASH {
                return Err(ConsensusError::TheMergeOmmerRootIsNotEmpty)
            }

            // Post-merge, the consensus layer is expected to perform checks such that the block
            // timestamp is a function of the slot. This is different from pre-merge, where blocks
            // are only allowed to be in the future (compared to the system's clock) by a certain
            // threshold.
            //
            // Block validation with respect to the parent should ensure that the block timestamp
            // is greater than its parent timestamp.

            // validate header extradata for all networks post merge
            validate_header_extradata(header)?;

            // mixHash is used instead of difficulty inside EVM
            // https://eips.ethereum.org/EIPS/eip-4399#using-mixhash-field-instead-of-difficulty
        } else {
            // TODO Consensus checks for old blocks:
            //  * difficulty, mix_hash & nonce aka PoW stuff
            // low priority as syncing is done in reverse order

            // Check if timestamp is in the future. Clock can drift but this can be consensus issue.
            let present_timestamp =
                SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

            if header.exceeds_allowed_future_timestamp(present_timestamp) {
                return Err(ConsensusError::TimestampIsInFuture {
                    timestamp: header.timestamp,
                    present_timestamp,
                })
            }

            validate_header_extradata(header)?;
        }

        Ok(())
    }
}

impl<ChainSpec: Send + Sync + EthChainSpec + EthereumHardforks + Debug> Consensus
    for TaikoSimpleBeaconConsensus<ChainSpec>
{
    fn validate_block_pre_execution(&self, _block: &SealedBlock) -> Result<(), ConsensusError> {
        Ok(())
    }

    fn validate_body_against_header(
        &self,
        _body: &BlockBody,
        _header: &SealedHeader,
    ) -> Result<(), ConsensusError> {
        Ok(())
    }
}

/// Validates the timestamp against the parent to make sure it is in the past or present.
#[inline]
pub fn validate_against_parent_timestamp_inclusive(
    header: &SealedHeader,
    parent: &SealedHeader,
) -> Result<(), ConsensusError> {
    if header.timestamp < parent.timestamp {
        return Err(ConsensusError::TimestampIsInPast {
            parent_timestamp: parent.timestamp,
            timestamp: header.timestamp,
        })
    }
    Ok(())
}
