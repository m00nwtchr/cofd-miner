use serde::{Deserialize, Serialize};

use crate::splat::werewolf::{Auspice, Renown};

use super::{ActionFields, Item};

#[derive(Debug, Serialize, Deserialize)]
pub struct Moon {
	pub auspice: Auspice,
	pub level: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Other {
	pub renown: Renown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Facet<T> {
	#[serde(flatten)]
	pub action: Option<ActionFields>,

	#[serde(flatten)]
	pub inner: T,
}

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub enum GiftKind {
	Moon,
	Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gift<T> {
	pub name: String,
	pub facets: Vec<Item<Facet<T>>>,
}
