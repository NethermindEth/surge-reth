//! Optimism block execution strategy.

use crate::{revm_spec, TaikoEvmConfig};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use alloy_consensus::{Header, Transaction as _};
use alloy_eips::eip7685::Requests;
use core::{cell::RefCell, fmt::Display};

use reth_chainspec::{EthereumHardfork, EthereumHardforks, Head};
use reth_consensus::ConsensusError;
use reth_ethereum_consensus::validate_block_post_execution;
use reth_evm::{
    execute::{
        balance_increment_state, BasicBatchExecutor, BlockExecutionError, BlockExecutionStrategy,
        BlockExecutionStrategyFactory, BlockExecutorProvider, BlockValidationError, ExecuteOutput,
        Executor, ProviderError,
    },
    state_change::post_block_balance_increments,
    system_calls::{OnStateHook, SystemCaller},
    ConfigureEvm, EnvExt, TxEnvOverrides,
};
use reth_evm_ethereum::dao_fork::{DAO_HARDFORK_BENEFICIARY, DAO_HARDKFORK_ACCOUNTS};
use reth_execution_types::{BlockExecutionInput, BlockExecutionOutput};
use reth_primitives::{
    BlockWithSenders, EthPrimitives, NodePrimitives, Receipt, TransactionSigned,
};
use reth_revm::{batch::BlockBatchRecord, Database, State};
use reth_taiko_chainspec::TaikoChainSpec;
use reth_taiko_consensus::{check_anchor_tx_by_spec_id, TaikoData};
use revm::JournaledState;
use revm_primitives::{
    db::DatabaseCommit, EVMError, EnvWithHandlerCfg, HashSet, ResultAndState, U256,
};
use tracing::debug;

use crate::alloc::string::ToString;

#[cfg(feature = "std")]
use flate2::{write::ZlibEncoder, Compression};

/// Factory for [`OpExecutionStrategy`].
#[derive(Debug, Clone)]
pub struct TaikoExecutionStrategyFactory<EvmConfig = TaikoEvmConfig> {
    /// The chainspec
    chain_spec: Arc<TaikoChainSpec>,
    /// How to create an EVM.
    evm_config: EvmConfig,
    /// Taiko Data
    taiko_data: TaikoData,
    /// Whether to skip invalid transactions (optimism). Default is false.
    optimistic: bool,
    /// Enable anchor transaction. Default is true.
    enable_anchor: bool,
}

impl TaikoExecutionStrategyFactory {
    /// Creates a new default taiko executor strategy factory.
    pub fn new(chain_spec: Arc<TaikoChainSpec>, taiko_data: TaikoData) -> Self {
        Self {
            chain_spec: chain_spec.clone(),
            evm_config: TaikoEvmConfig::new(chain_spec),
            taiko_data,
            optimistic: false,
            enable_anchor: true,
        }
    }
}

impl<EvmConfig> TaikoExecutionStrategyFactory<EvmConfig> {
    /// Enable skip invalid transactions (optimism). Default is false.
    pub fn with_optimistic(mut self, optimistic: bool) -> Self {
        self.optimistic = optimistic;
        self
    }

    /// Enable anchor transaction. Default is true.
    pub fn with_enable_anchor(mut self, enable_anchor: bool) -> Self {
        self.enable_anchor = enable_anchor;
        self
    }
}

impl<EvmConfig> BlockExecutionStrategyFactory for TaikoExecutionStrategyFactory<EvmConfig>
where
    EvmConfig: Clone
        + Unpin
        + Sync
        + Send
        + 'static
        + ConfigureEvm<Header = alloy_consensus::Header, Transaction = TransactionSigned>,
{
    type Primitives = EthPrimitives;
    type Strategy<DB: Database<Error: Into<ProviderError> + Display>> =
        TaikoExecutionStrategy<DB, EvmConfig>;

    fn create_strategy<DB>(&self, db: DB) -> Self::Strategy<DB>
    where
        DB: Database<Error: Into<ProviderError> + Display>,
    {
        let state =
            State::builder().with_database(db).with_bundle_update().without_state_clear().build();
        TaikoExecutionStrategy::new(
            state,
            self.chain_spec.clone(),
            self.evm_config.clone(),
            self.taiko_data.clone(),
            self.optimistic,
            self.enable_anchor,
        )
    }
}

/// Block execution strategy for Optimism.
#[allow(missing_debug_implementations)]
pub struct TaikoExecutionStrategy<DB, EvmConfig>
where
    EvmConfig: Clone,
{
    /// The chainspec
    chain_spec: Arc<TaikoChainSpec>,
    /// How to create an EVM.
    evm_config: EvmConfig,
    /// Optional overrides for the transactions environment.
    tx_env_overrides: Option<Box<dyn TxEnvOverrides>>,
    /// Current state for block execution.
    state: State<DB>,
    /// Utility to call system smart contracts.
    system_caller: SystemCaller<EvmConfig, TaikoChainSpec>,
    /// Taiko data
    taiko_data: TaikoData,
    /// Whether to skip invalid transactions (optimism). Default is false.
    optimistic: bool,
    /// Enable anchor transaction. Default is true.
    enable_anchor: bool,
}

impl<DB, EvmConfig> TaikoExecutionStrategy<DB, EvmConfig>
where
    EvmConfig: Clone,
{
    /// Creates a new [`TaikoExecutionStrategy`]
    pub fn new(
        state: State<DB>,
        chain_spec: Arc<TaikoChainSpec>,
        evm_config: EvmConfig,
        taiko_data: TaikoData,
        optimistic: bool,
        enable_anchor: bool,
    ) -> Self {
        let system_caller = SystemCaller::new(evm_config.clone(), chain_spec.clone());
        Self {
            state,
            chain_spec,
            evm_config,
            system_caller,
            tx_env_overrides: None,
            taiko_data,
            optimistic,
            enable_anchor,
        }
    }
}

impl<DB, EvmConfig> TaikoExecutionStrategy<DB, EvmConfig>
where
    DB: Database<Error: Into<ProviderError> + Display>,
    EvmConfig: ConfigureEvm<Header = alloy_consensus::Header>,
{
    /// Configures a new evm configuration and block environment for the given block.
    ///
    /// Caution: this does not initialize the tx environment.
    fn evm_env_for_block(&self, header: &Header, total_difficulty: U256) -> EnvWithHandlerCfg {
        let (cfg, block_env) = self.evm_config.cfg_and_block_env(header, total_difficulty);
        EnvWithHandlerCfg::new_with_cfg_env(cfg, block_env, Default::default())
    }
}

impl<DB, EvmConfig> BlockExecutionStrategy for TaikoExecutionStrategy<DB, EvmConfig>
where
    DB: Database<Error: Into<ProviderError> + Display>,
    EvmConfig: ConfigureEvm<Header = alloy_consensus::Header, Transaction = TransactionSigned>,
{
    type DB = DB;
    type Error = BlockExecutionError;

    type Primitives = EthPrimitives;

    fn init(&mut self, tx_env_overrides: Box<dyn TxEnvOverrides>) {
        self.tx_env_overrides = Some(tx_env_overrides);
    }

    fn apply_pre_execution_changes(
        &mut self,
        block: &BlockWithSenders,
        total_difficulty: U256,
    ) -> Result<(), Self::Error> {
        // Set state clear flag if the block is after the Spurious Dragon hardfork.
        let state_clear_flag =
            (*self.chain_spec).is_spurious_dragon_active_at_block(block.header.number);
        self.state.set_state_clear_flag(state_clear_flag);

        let env = self.evm_env_for_block(&block.header, total_difficulty);
        let mut evm = self.evm_config.evm_with_env(&mut self.state, env);

        self.system_caller.apply_pre_execution_changes(&block.block, &mut evm)?;

        Ok(())
    }

    fn execute_transactions(
        &mut self,
        input: BlockExecutionInput<'_, BlockWithSenders>,
    ) -> Result<ExecuteOutput<Receipt>, Self::Error> {
        let BlockExecutionInput { block, total_difficulty } = input;

        let env = self.evm_env_for_block(&block.header, total_difficulty);
        let mut evm = self.evm_config.evm_with_env(&mut self.state, env);
        let mut cumulative_gas_used = 0;
        let mut receipts = Vec::with_capacity(block.body.transactions.len());
        let mut skipped_list = Vec::with_capacity(block.body.transactions.len());
        let treasury = self.chain_spec.treasury();

        for (idx, (sender, transaction)) in block.transactions_with_sender().enumerate() {
            let is_anchor = idx == 0 && self.enable_anchor;

            // verify the anchor tx
            if is_anchor {
                let spec_id = revm_spec(
                    &self.chain_spec,
                    &Head { number: block.number, ..Default::default() },
                );

                check_anchor_tx_by_spec_id(
                    spec_id,
                    transaction,
                    sender,
                    block,
                    self.taiko_data.clone(),
                )
                .map_err(|e| BlockValidationError::AnchorValidation { message: e.to_string() })?;
            }

            // The sum of the transaction’s gas limit, Tg, and the gas utilized in this block prior,
            // must be no greater than the block’s gasLimit.
            let block_available_gas = block.header.gas_limit - cumulative_gas_used;
            if transaction.gas_limit() > block_available_gas {
                if !is_anchor && self.optimistic {
                    debug!(target: "taiko::executor", hash = ?transaction.hash(), want = ?transaction.gas_limit(), got = block_available_gas, "Invalid gas limit for tx");
                    skipped_list.push(idx);
                    continue;
                }
                return Err(BlockValidationError::TransactionGasLimitMoreThanAvailableBlockGas {
                    transaction_gas_limit: transaction.gas_limit(),
                    block_available_gas,
                }
                .into());
            }

            self.evm_config.fill_tx_env(
                evm.tx_mut(),
                transaction,
                *sender,
                Some(EnvExt {
                    is_anchor,
                    block_number: block.number,
                    extra_data: &block.extra_data,
                }),
            );

            if let Some(tx_env_overrides) = &mut self.tx_env_overrides {
                tx_env_overrides.apply(evm.tx_mut());
            }

            // Execute transaction.
            let result_and_state = match evm.transact().map_err(move |err| {
                let new_err = err.map_db_err(|e| e.into());
                // Ensure hash is calculated for error log, if not already done
                BlockValidationError::EVM {
                    hash: transaction.recalculate_hash(),
                    error: Box::new(new_err),
                }
            }) {
                Ok(res) => res,
                Err(err) => {
                    // Clear the state for the next tx
                    evm.context.evm.journaled_state = JournaledState::new(
                        evm.context.evm.journaled_state.spec,
                        HashSet::default(),
                    );
                    if self.optimistic {
                        // Clear the state for the next tx
                        // evm.context.evm.journaled_state = JournaledState::new(
                        //     evm.context.evm.journaled_state.spec,
                        //     HashSet::default(),
                        // );
                        debug!(target: "taiko::executor", hash = ?transaction.hash(), error = ?err, "Invalid execute for tx");
                        skipped_list.push(idx);
                        continue;
                    }

                    if is_anchor {
                        return Err(BlockExecutionError::Validation(err));
                    }

                    // only continue for invalid tx errors, not db errors (because those can be
                    // manipulated by the prover)
                    match err {
                        BlockValidationError::EVM { hash, error } => match *error {
                            EVMError::Transaction(invalid_transaction) => {
                                println!("Invalid tx at {}: {:?}", idx, invalid_transaction);
                                // skip the tx
                                continue;
                            }
                            _ => {
                                // any other error is not allowed
                                return Err(BlockExecutionError::Validation(
                                    BlockValidationError::EVM { hash, error },
                                ));
                            }
                        },
                        _ => {
                            // Any other type of error is not allowed
                            return Err(BlockExecutionError::Validation(err));
                        }
                    }
                }
            };

            self.system_caller.on_state(&result_and_state.state);
            let ResultAndState { result, state } = result_and_state;
            evm.db_mut().commit(state);

            // append gas used
            cumulative_gas_used += result.gas_used();

            // Push transaction changeset and calculate header bloom filter for receipt.
            receipts.push(
                #[allow(clippy::needless_update)] // side-effect of optimism fields
                Receipt {
                    tx_type: transaction.tx_type(),
                    // Success flag was added in `EIP-658: Embedding transaction status code in
                    // receipts`.
                    success: result.is_success(),
                    cumulative_gas_used,
                    // convert to reth log
                    logs: result.into_logs(),
                    ..Default::default()
                },
            );
        }
        Ok(ExecuteOutput { receipts, gas_used: cumulative_gas_used, skipped_list })
    }

    fn apply_post_execution_changes(
        &mut self,
        block: &BlockWithSenders,
        total_difficulty: U256,
        receipts: &[Receipt],
    ) -> Result<Requests, Self::Error> {
        let env = self.evm_env_for_block(&block.header, total_difficulty);
        let mut evm = self.evm_config.evm_with_env(&mut self.state, env);

        let requests = if self.chain_spec.is_prague_active_at_timestamp(block.timestamp) {
            // Collect all EIP-6110 deposits
            let deposit_requests = reth_evm_ethereum::eip6110::parse_deposits_from_receipts(
                &self.chain_spec,
                receipts,
            )?;

            let mut requests = Requests::default();

            if !deposit_requests.is_empty() {
                requests.push_request(core::iter::once(0).chain(deposit_requests).collect());
            }

            requests.extend(self.system_caller.apply_post_execution_changes(&mut evm)?);
            requests
        } else {
            Requests::default()
        };
        drop(evm);

        let mut balance_increments =
            post_block_balance_increments(&self.chain_spec, &block.block, total_difficulty);

        // Irregular state change at Ethereum DAO hardfork
        if self.chain_spec.fork(EthereumHardfork::Dao).transitions_at_block(block.number) {
            // drain balances from hardcoded addresses.
            let drained_balance: u128 = self
                .state
                .drain_balances(DAO_HARDKFORK_ACCOUNTS)
                .map_err(|_| BlockValidationError::IncrementBalanceFailed)?
                .into_iter()
                .sum();

            // return balance to DAO beneficiary.
            *balance_increments.entry(DAO_HARDFORK_BENEFICIARY).or_default() += drained_balance;
        }
        // increment balances
        self.state
            .increment_balances(balance_increments.clone())
            .map_err(|_| BlockValidationError::IncrementBalanceFailed)?;
        // call state hook with changes due to balance increments.
        let balance_state = balance_increment_state(&balance_increments, &mut self.state)?;
        self.system_caller.on_state(&balance_state);

        Ok(requests)
    }

    fn state_ref(&self) -> &State<DB> {
        &self.state
    }

    fn state(self) -> State<DB> {
        self.state
    }

    fn state_mut(&mut self) -> &mut State<DB> {
        &mut self.state
    }

    fn with_state_hook(&mut self, hook: Option<Box<dyn OnStateHook>>) {
        self.system_caller.with_state_hook(hook);
    }

    fn validate_block_post_execution(
        &self,
        block: &BlockWithSenders,
        receipts: &[Receipt],
        requests: &Requests,
    ) -> Result<(), ConsensusError> {
        validate_block_post_execution(block, &self.chain_spec.clone(), receipts, requests)
    }
}

/// A taiko block executor that uses a [`BlockExecutionStrategy`] to
/// execute blocks.
#[expect(missing_debug_implementations)]
pub struct TaikoBlockExecutor<S, DB> {
    /// Block execution strategy.
    pub(crate) strategy: S,
    /// Post execution state, used to get the state after execution.
    pub post_execute_state: Arc<RefCell<Option<State<DB>>>>,
}

impl<S, DB> TaikoBlockExecutor<S, DB> {
    /// Creates a new `TaikoBlockExecutor` with the given strategy.
    pub fn new(strategy: S) -> Self {
        Self { strategy, post_execute_state: Arc::default() }
    }
}

impl<S, DB> Executor<DB> for TaikoBlockExecutor<S, DB>
where
    S: BlockExecutionStrategy<DB = DB>,
    DB: Database<Error: Into<ProviderError> + Display>,
{
    type Input<'a> =
        BlockExecutionInput<'a, BlockWithSenders<<S::Primitives as NodePrimitives>::Block>>;
    type Output = BlockExecutionOutput<<S::Primitives as NodePrimitives>::Receipt>;
    type Error = S::Error;

    fn init(&mut self, env_overrides: Box<dyn TxEnvOverrides>) {
        self.strategy.init(env_overrides);
    }

    fn execute(mut self, input: Self::Input<'_>) -> Result<Self::Output, Self::Error> {
        let BlockExecutionInput { block, total_difficulty, .. } = input;

        self.strategy.apply_pre_execution_changes(block, total_difficulty)?;
        let ExecuteOutput { receipts, gas_used, skipped_list } =
            self.strategy.execute_transactions(input)?;
        let requests =
            self.strategy.apply_post_execution_changes(block, total_difficulty, &receipts)?;

        let state = self.strategy.finish();

        *self.post_execute_state.borrow_mut() = Some(self.strategy.state());

        Ok(BlockExecutionOutput { state, receipts, requests, gas_used, skipped_list })
    }

    fn execute_with_state_closure<F>(
        mut self,
        input: Self::Input<'_>,
        mut state: F,
    ) -> Result<Self::Output, Self::Error>
    where
        F: FnMut(&State<DB>),
    {
        let BlockExecutionInput { block, total_difficulty, .. } = input;

        self.strategy.apply_pre_execution_changes(block, total_difficulty)?;
        let ExecuteOutput { receipts, gas_used, skipped_list } =
            self.strategy.execute_transactions(input)?;
        let requests =
            self.strategy.apply_post_execution_changes(block, total_difficulty, &receipts)?;

        state(self.strategy.state_ref());

        let state = self.strategy.finish();

        *self.post_execute_state.borrow_mut() = Some(self.strategy.state());

        Ok(BlockExecutionOutput { state, receipts, requests, gas_used, skipped_list })
    }

    fn execute_with_state_hook<H>(
        mut self,
        input: Self::Input<'_>,
        state_hook: H,
    ) -> Result<Self::Output, Self::Error>
    where
        H: OnStateHook + 'static,
    {
        let BlockExecutionInput { block, total_difficulty, .. } = input;

        self.strategy.with_state_hook(Some(Box::new(state_hook)));

        self.strategy.apply_pre_execution_changes(block, total_difficulty)?;
        let ExecuteOutput { receipts, gas_used, skipped_list } =
            self.strategy.execute_transactions(input)?;
        let requests =
            self.strategy.apply_post_execution_changes(block, total_difficulty, &receipts)?;

        let state = self.strategy.finish();

        *self.post_execute_state.borrow_mut() = Some(self.strategy.state());

        Ok(BlockExecutionOutput { state, receipts, requests, gas_used, skipped_list })
    }
}

impl<S, DB> TaikoBlockExecutor<S, DB>
where
    S: BlockExecutionStrategy<DB = DB>,
    DB: Database<Error: Into<ProviderError> + Display>,
{
    /// Consumes the type, executes the block and returns the output with the post execution state.
    ///
    /// # Returns
    /// The output of the block execution, state
    pub fn execute_and_get_state(
        self,
        input: <Self as Executor<DB>>::Input<'_>,
    ) -> Result<(<Self as Executor<DB>>::Output, State<DB>), <Self as Executor<DB>>::Error> {
        let state = self.post_execute_state.clone();
        let output = self.execute(input)?;

        // Getting the post execute state and replacing it with None
        let state = state.replace(None).expect("State should be set");
        Ok((output, state))
    }
}

/// A taiko block executor provider that can create executors using a strategy factory.
#[derive(Debug)]
pub struct TaikoBlockExecutorProvider<F> {
    strategy_factory: F,
}

impl<F> Clone for TaikoBlockExecutorProvider<F>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Self { strategy_factory: self.strategy_factory.clone() }
    }
}

impl<F> TaikoBlockExecutorProvider<F> {
    /// Creates a new `TaikoBlockExecutorProvider` with the given strategy factory.
    pub const fn new(strategy_factory: F) -> Self {
        Self { strategy_factory }
    }
}

impl<F> BlockExecutorProvider for TaikoBlockExecutorProvider<F>
where
    F: BlockExecutionStrategyFactory,
{
    type Primitives = F::Primitives;

    type Executor<DB: Database<Error: Into<ProviderError> + Display>> =
        TaikoBlockExecutor<F::Strategy<DB>, DB>;

    type BatchExecutor<DB: Database<Error: Into<ProviderError> + Display>> =
        BasicBatchExecutor<F::Strategy<DB>>;

    fn executor<DB>(&self, db: DB) -> Self::Executor<DB>
    where
        DB: Database<Error: Into<ProviderError> + Display>,
    {
        let strategy = self.strategy_factory.create_strategy(db);
        TaikoBlockExecutor::new(strategy)
    }

    fn batch_executor<DB>(&self, db: DB) -> Self::BatchExecutor<DB>
    where
        DB: Database<Error: Into<ProviderError> + Display>,
    {
        let strategy = self.strategy_factory.create_strategy(db);
        let batch_record = BlockBatchRecord::default();
        BasicBatchExecutor::new(strategy, batch_record)
    }
}

/// Helper type with backwards compatible methods to obtain executor providers.
#[derive(Debug)]
pub struct TaikoExecutorProviderBuilder(TaikoExecutionStrategyFactory);

impl TaikoExecutorProviderBuilder {
    /// Creates a new default taiko executor strategy factory.
    pub fn new(chain_spec: Arc<TaikoChainSpec>, taiko_data: TaikoData) -> Self {
        TaikoExecutorProviderBuilder(TaikoExecutionStrategyFactory::new(chain_spec, taiko_data))
    }

    /// Enable skip invalid transactions (optimism). Default is false.
    pub fn with_optimistic(mut self, optimistic: bool) -> Self {
        self.0 = self.0.with_optimistic(optimistic);
        self
    }

    /// Enable anchor transaction. Default is true.
    pub fn with_enable_anchor(mut self, enable_anchor: bool) -> Self {
        self.0 = self.0.with_enable_anchor(enable_anchor);
        self
    }

    /// Creates a new default taiko executor strategy factory.
    pub fn build(self) -> TaikoBlockExecutorProvider<TaikoExecutionStrategyFactory> {
        TaikoBlockExecutorProvider::new(self.0)
    }
}

#[cfg(feature = "std")]
/// Encode and compress a list of transactions.
pub fn encode_and_compress_tx_list<T: alloy_rlp::Encodable>(
    txs: &Vec<T>,
) -> std::io::Result<Vec<u8>> {
    let encoded_buf = alloy_rlp::encode(txs);
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    <ZlibEncoder<_> as std::io::Write>::write_all(&mut encoder, &encoded_buf)?;
    encoder.finish()
}
