use serde::{Deserialize, Serialize};

use self::{gift::FacetKind, merit::Merit};
use crate::{book::BookReference, dice_pool::DicePool};

pub mod gift;
pub mod merit;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionFields {
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub cost: Vec<String>,
	pub dice_pool: Option<DicePool>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub action: Vec<String>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub duration: Vec<String>,
}

pub enum ItemKind {
	Merit(Item<Merit>),
	Facet(FacetKind),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item<T> {
	pub name: String,
	pub reference: BookReference,

	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub description: Vec<String>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub effects: Vec<String>,

	#[serde(flatten)]
	pub inner: T,
}
