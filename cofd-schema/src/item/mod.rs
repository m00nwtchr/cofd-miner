use serde::{Deserialize, Serialize};

use self::merit::Merit;
use crate::{
	book::{BookReference, MoonGift, OtherGift},
	dice_pool::DicePool,
};

pub mod gift;
pub mod merit;
pub mod spell;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RollResults {
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub exceptional_success: Vec<String>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub success: Vec<String>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub failure: Vec<String>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub dramatic_failure: Vec<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionFields {
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub cost: Vec<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub dice_pool: Option<DicePool>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub action: Vec<String>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub duration: Vec<String>,

	pub roll_results: RollResults,
}

pub enum ItemKind {
	Merit(Item<Merit>),
	MoonGift(MoonGift),
	OtherGift(OtherGift),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
