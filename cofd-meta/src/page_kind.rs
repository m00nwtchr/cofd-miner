#![allow(clippy::tabs_in_doc_comments)]
use cofd_schema::item::gift::GiftKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum PageKind {
	Merit(
		/**
		 * Additional pre-requisites
		 */
		Option<String>,
	),
	MageSpell,
	Gift(GiftKind),
}

impl Default for PageKind {
	fn default() -> Self {
		Self::Merit(None)
	}
}
