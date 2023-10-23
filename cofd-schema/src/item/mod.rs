use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use strum::EnumString;

use crate::{
	dice_pool::DicePool,
	dot_range::DotRange,
	prerequisites::{Prerequisite, Prerequisites},
};
use merit::MeritTag;

pub mod merit;

#[derive(
	Debug, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq, EnumString,
)]
#[strum(ascii_case_insensitive)]
#[serde(rename_all = "camelCase")]
pub enum ItemProp {
	// Description,
	DotRating,
	Tags,

	#[strum(serialize = "Prerequisites", serialize = "Prerequisite")]
	Prerequisites,
	#[strum(to_string = "Style Tags", serialize = "Style Tag")]
	StyleTags,
	Cost,
	#[strum(to_string = "Dice Pool")]
	DicePool,
	Action,
	Duration,
	#[strum(serialize = "Effects", serialize = "Effect")]
	Effects,
	#[strum(serialize = "Drawbacks", serialize = "Drawback")]
	Drawbacks,
	#[strum(serialize = "Notes", serialize = "Note")]
	Notes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropValue {
	Vec(Vec<String>),
	Bool(bool),
	DotRange(DotRange),
	DicePool(DicePool),
	Prerequisites(Prerequisites),
	Tags(Vec<MeritTag>),
}

impl PropValue {
	pub fn insert(&mut self, index: usize, element: String) {
		if let PropValue::Vec(vec) = self {
			vec.insert(index, element)
		}
	}
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SubItem {
	pub name: String,
	pub description: Vec<String>,
	#[serde(default, skip_serializing_if = "crate::is_empty_map", flatten)]
	pub properties: BTreeMap<ItemProp, PropValue>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Item {
	pub name: String,
	pub page: usize,
	#[serde(default, skip_serializing_if = "crate::is_empty")]
	pub children: Vec<SubItem>,
	pub description: Vec<String>,
	#[serde(default, skip_serializing_if = "crate::is_empty_map", flatten)]
	pub properties: BTreeMap<ItemProp, PropValue>,
}
