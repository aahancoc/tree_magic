//! Read magic file bundled in crate

use petgraph::prelude::*;
use fnv::FnvHashMap;
use crate::MIME;
use super::MagicRule;

/// Preload alias list
lazy_static! {
	static ref ALIASES: FnvHashMap<MIME, MIME> = {
		init::get_aliaslist()
	};
}

/// Load magic file before anything else.
lazy_static! {
    static ref ALLRULES: FnvHashMap<MIME, DiGraph<MagicRule, u32>> = {
        super::ruleset::from_u8(include_bytes!("magic")).unwrap_or(FnvHashMap::default())
    };
}

pub mod init;
pub mod check;
