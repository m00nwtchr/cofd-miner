use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use super::{ActionFields, Item};

#[derive(Debug, Serialize, Deserialize)]
pub struct Moon {
	pub level: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Other {
	pub renown: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Facet<T> {
	#[serde(flatten)]
	pub action: Option<ActionFields>,

	pub inner: T,
}

#[derive(EnumDiscriminants)]
#[strum_discriminants(derive(Serialize, Deserialize, Hash))]
#[strum_discriminants(name(GiftKind))]
pub enum FacetKind {
	Moon(Item<Facet<Moon>>),
	Other(Item<Facet<Other>>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gift<T> {
	pub name: String,
	pub facets: Vec<Item<Facet<T>>>,
}
