use alloy_eips::eip7685::Requests;
use alloy_primitives::U256;
use reth_primitives::BlockWithSenders;
use revm::db::BundleState;

/// A helper type for ethereum block inputs that consists of a block and the total difficulty.
#[derive(Debug, Clone, Copy)]
pub struct BlockExecutionInput<'a, Block> {
    /// The block to execute.
    pub block: &'a Block,
    /// The total difficulty of the block.
    pub total_difficulty: U256,
}

impl<'a, Block> BlockExecutionInput<'a, Block> {
    /// Creates a new input.
    pub const fn new(block: &'a Block, total_difficulty: U256) -> Self {
        Self { block, total_difficulty }
    }
}

impl<'a, Block> From<(&'a Block, U256)> for BlockExecutionInput<'a, Block> {
    fn from((block, total_difficulty): (&'a Block, U256)) -> Self {
        Self::new(block, total_difficulty)
    }
}

/// The output of an ethereum block.
///
/// Contains the state changes, transaction receipts, and total gas used in the block.
#[derive(Debug, Clone)]
pub struct BlockExecutionOutput<T> {
    /// The changed state of the block after execution.
    pub state: BundleState,
    /// All the receipts of the transactions in the block.
    pub receipts: Vec<T>,
    /// All the EIP-7685 requests in the block.
    pub requests: Requests,
    /// The total gas used by the block.
    pub gas_used: u64,
    /// The skipped transactions when optimistic set to true.
    pub skipped_list: Vec<usize>,
}

impl<T> BlockExecutionOutput<T> {
    /// Remote the skipped transactions from the block.
    pub fn apply_skip(&self, block: &mut BlockWithSenders) {
        retain_with_index(&mut block.senders, |i, _| !self.skipped_list.contains(&i));
        retain_with_index(&mut block.body.transactions, |i, _| !self.skipped_list.contains(&i));
    }
}

fn retain_with_index<T, F>(slice: &mut Vec<T>, mut f: F)
where
    F: FnMut(usize, &T) -> bool,
{
    let mut i = 0;
    slice.retain(|x| {
        let retain = f(i, x);
        i += 1;
        retain
    });
}
