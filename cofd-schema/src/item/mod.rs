use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{dot_range::DotRange, prerequisites::Prerequisite};

pub mod merit;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ItemProp {
	Description,
	DotRating,
	Tags,

	Prerequisites,
	StyleTags,
	Cost,
	DicePool,
	Action,
	Duration,
	Effects,
	Drawbacks,
	Notes,
}

impl ItemProp {
	pub fn by_name(str: &str) -> Option<Self> {
		match str.to_lowercase().as_str() {
			"prerequisite" | "prerequisites" => Some(Self::Prerequisites),
			"style tag" | "style tags" => Some(Self::StyleTags),
			"cost" => Some(Self::Cost),
			"dice pool" => Some(Self::DicePool),
			"action" => Some(Self::Action),
			"duration" => Some(Self::Duration),
			"effect" | "effects" => Some(Self::Effects),
			"drawback" | "drawbacks" => Some(Self::Drawbacks),
			"note" | "notes" => Some(Self::Notes),
			_ => None,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropValue {
	Vec(Vec<String>),
	Bool(bool),
	DotRange(DotRange),
	Prerequisites(Vec<Prerequisite>),
}

impl PropValue {
	pub fn insert(&mut self, index: usize, element: String) {
		if let PropValue::Vec(vec) = self {
			vec.insert(index, element)
		}
	}
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SubItem {
	pub name: String,
	pub desc: Vec<String>,
	#[serde(default, skip_serializing_if = "crate::is_empty_map")]
	pub props: HashMap<ItemProp, PropValue>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Item {
	pub name: String,
	#[serde(default, skip_serializing_if = "crate::is_empty")]
	pub children: Vec<SubItem>,
	pub desc: Vec<String>,
	#[serde(default, skip_serializing_if = "crate::is_empty_map")]
	pub props: HashMap<ItemProp, PropValue>,
}
