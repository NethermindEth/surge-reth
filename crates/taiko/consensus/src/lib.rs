//! Beacon consensus implementation.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

mod anchor;
#[cfg(feature = "beacon-consensus")]
mod taiko_beacon_consensus;
mod taiko_simple_consensus;

pub use anchor::*;
#[cfg(feature = "beacon-consensus")]
pub use taiko_beacon_consensus::*;
pub use taiko_simple_consensus::*;
