#![allow(clippy::tabs_in_doc_comments)]
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
}

impl Default for PageKind {
	fn default() -> Self {
		Self::Merit(None)
	}
}
