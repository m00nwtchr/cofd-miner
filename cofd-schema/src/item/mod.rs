use serde::{Deserialize, Serialize};

use self::merit::MeritSubItem;
use crate::{dice_pool::DicePool, dot_range::DotRange, prerequisites::Prerequisites};
use merit::MeritTag;

pub mod merit;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionFields {
	#[serde(default, skip_serializing_if = "crate::is_empty")]
	pub cost: Vec<String>,
	pub dice_pool: DicePool,
	#[serde(default, skip_serializing_if = "crate::is_empty")]
	pub action: Vec<String>,
	#[serde(default, skip_serializing_if = "crate::is_empty")]
	pub duration: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ItemType {
	#[serde(rename_all = "camelCase")]
	Merit {
		dot_rating: DotRange,
		#[serde(default, skip_serializing_if = "crate::is_empty")]
		prerequisites: Prerequisites,
		#[serde(default, skip_serializing_if = "crate::is_empty")]
		style_tags: Vec<MeritTag>,
		#[serde(default, skip_serializing_if = "crate::is_empty")]
		drawbacks: Vec<String>,

		#[serde(default, skip_serializing_if = "crate::is_empty")]
		children: Vec<MeritSubItem>,

		#[serde(flatten)]
		action: Option<ActionFields>,

		#[serde(default, skip_serializing_if = "crate::is_empty")]
		notes: Vec<String>,
	},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
	pub name: String,
	pub page: usize,

	#[serde(default, skip_serializing_if = "crate::is_empty")]
	pub description: Vec<String>,
	#[serde(default, skip_serializing_if = "crate::is_empty")]
	pub effects: Vec<String>,

	#[serde(flatten)]
	pub inner: ItemType,
}
