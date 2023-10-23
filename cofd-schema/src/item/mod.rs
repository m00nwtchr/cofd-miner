use std::{collections::BTreeMap, fmt::Display};

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::{
	dice_pool::DicePool,
	dot_range::DotRange,
	prerequisites::{Prerequisite, Prerequisites},
};
use merit::MeritTag;

pub mod merit;

#[derive(
	Debug, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq, EnumString, Display,
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

impl Display for PropValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PropValue::Vec(vec) => vec.join("\n").fmt(f),
			PropValue::Bool(b) => b.fmt(f),
			PropValue::DotRange(dr) => dr.fmt(f),
			PropValue::DicePool(d) => d.fmt(f),
			PropValue::Prerequisites(p) => p.fmt(f),
			PropValue::Tags(_) => todo!(),
		}
	}
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
