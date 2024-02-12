use serde::{Deserialize, Serialize};

use crate::splat::werewolf::{Auspice, Renown};

use super::{ActionFields, Item};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Moon {
	pub auspice: Auspice,
	pub level: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Other {
	pub renown: Renown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Facet<T> {
	#[serde(flatten)]
	pub action: Option<ActionFields>,

	#[serde(flatten)]
	pub inner: T,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Copy)]
pub enum GiftKind {
	Moon,
	Shadow,
	Wolf,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Gift<T> {
	pub name: String,
	pub facets: Vec<Item<Facet<T>>>,
	pub kind: GiftKind,
}
