//! Collection of traits and trait implementations for common database operations.
//!
//! ## Feature Flags
//!
//! - `test-utils`: Export utilities for testing

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
//#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

/// Various provider traits.
#[cfg(feature = "not-zkvm")]
mod traits;
#[cfg(feature = "not-zkvm")]
pub use traits::*;

/// Provider trait implementations.
#[cfg(feature = "not-zkvm")]
pub mod providers;
#[cfg(feature = "not-zkvm")]
pub use providers::{
    DatabaseProvider, DatabaseProviderRO, DatabaseProviderRW, HistoricalStateProvider,
    HistoricalStateProviderRef, LatestStateProvider, LatestStateProviderRef, ProviderFactory,
    StaticFileAccess, StaticFileWriter,
};

#[cfg(any(test, feature = "test-utils"))]
/// Common test helpers for mocking the Provider.
pub mod test_utils;

/// Re-export provider error.
pub use reth_storage_errors::provider::{ProviderError, ProviderResult};

pub use reth_execution_types::*;

pub mod bundle_state;
pub use bundle_state::{StateChanges, StateReverts};
pub use revm::db::states::OriginalValuesKnown;

#[cfg(feature = "not-zkvm")]
pub(crate) fn to_range<R: std::ops::RangeBounds<u64>>(bounds: R) -> std::ops::Range<u64> {
    let start = match bounds.start_bound() {
        std::ops::Bound::Included(&v) => v,
        std::ops::Bound::Excluded(&v) => v + 1,
        std::ops::Bound::Unbounded => 0,
    };

    let end = match bounds.end_bound() {
        std::ops::Bound::Included(&v) => v + 1,
        std::ops::Bound::Excluded(&v) => v,
        std::ops::Bound::Unbounded => u64::MAX,
    };

    start..end
}
