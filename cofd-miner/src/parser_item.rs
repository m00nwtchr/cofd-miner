use std::{collections::BTreeMap, str::FromStr};

use cofd_meta_schema::PageKind;
use serde::{Deserialize, Serialize};
use strum::EnumString;

use cofd_schema::{
	dice_pool::DicePool,
	item::{
		merit::{MeritSubItem, MeritTag},
		ActionFields, Item, ItemType,
	},
	prelude::DotRange,
	prerequisites::{Prerequisite, Prerequisites},
};

#[derive(
	Debug, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq, EnumString,
)]
#[strum(ascii_case_insensitive)]
#[serde(rename_all = "camelCase")]
pub enum ItemProp {
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
			vec.insert(index, element);
		}
	}
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ParserSubItem {
	pub name: String,
	pub description: Vec<String>,
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty", flatten)]
	pub properties: BTreeMap<ItemProp, PropValue>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ParserItem {
	pub name: String,
	pub page: usize,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub children: Vec<ParserSubItem>,
	pub description: Vec<String>,
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty", flatten)]
	pub properties: BTreeMap<ItemProp, PropValue>,
}

fn convert_prerequisites(vec: Vec<String>) -> Prerequisites {
	let mut prereqs = Vec::new();

	for str in vec {
		if let Ok(prereq) = Prerequisite::from_str(&str) {
			prereqs.push(prereq);
		}
	}

	prereqs.into()
}

fn convert_tags(vec: Vec<String>) -> Vec<MeritTag> {
	let mut tags = Vec::new();

	for str in vec {
		if let Ok(prereq) = MeritTag::from_str(&str) {
			tags.push(prereq);
		}
	}

	tags
}

#[warn(clippy::needless_pass_by_value)]
fn convert_dice_pool(vec: Vec<String>) -> Option<DicePool> {
	if vec.len() == 1 {
		let str = vec.first().unwrap();

		if let Ok(pool) = DicePool::from_str(str) {
			return Some(pool);
		}
	}
	None
}

#[allow(clippy::too_many_lines)]
pub(crate) fn convert_item(kind: &PageKind, item: ParserItem) -> Item {
	let mut properties = item.properties;
	Item {
		name: item.name,
		page: item.page,
		description: item.description,
		effects: properties
			.remove(&ItemProp::Effects)
			.and_then(|v| match v {
				PropValue::Vec(ve) => Some(ve),
				_ => None,
			})
			.unwrap_or_default(),
		inner: match kind {
			PageKind::Merit(_) => ItemType::Merit {
				dot_rating: properties
					.remove(&ItemProp::DotRating)
					.and_then(|v| match v {
						PropValue::DotRange(dr) => Some(dr),
						_ => None,
					})
					.unwrap(),
				prerequisites: properties
					.remove(&ItemProp::Prerequisites)
					.and_then(|v| match v {
						PropValue::Vec(vec) => Some(convert_prerequisites(vec)),
						_ => None,
					})
					.unwrap_or_default(),
				style_tags: properties
					.remove(&ItemProp::StyleTags)
					.and_then(|v| match v {
						PropValue::Vec(vec) => Some(convert_tags(vec)),
						_ => None,
					})
					.unwrap_or_default(),
				drawbacks: properties
					.remove(&ItemProp::Drawbacks)
					.and_then(|v| match v {
						PropValue::Vec(vec) => Some(vec),
						_ => None,
					})
					.unwrap_or_default(),
				children: item
					.children
					.into_iter()
					.map(|mut item| MeritSubItem {
						name: item.name,
						description: item.description,
						prerequisites: item
							.properties
							.remove(&ItemProp::Prerequisites)
							.and_then(|v| match v {
								PropValue::Vec(vec) => Some(convert_prerequisites(vec)),
								_ => None,
							})
							.unwrap_or_default(),
						dot_rating: item
							.properties
							.remove(&ItemProp::DotRating)
							.and_then(|v| match v {
								PropValue::DotRange(dr) => Some(dr),
								_ => None,
							})
							.unwrap(),
						drawbacks: item
							.properties
							.remove(&ItemProp::Drawbacks)
							.and_then(|v| match v {
								PropValue::Vec(vec) => Some(vec),
								_ => None,
							})
							.unwrap_or_default(),
					})
					.collect(),
				action: {
					let cost = properties.remove(&ItemProp::Cost).and_then(|i| match i {
						PropValue::Vec(v) => Some(v),
						_ => None,
					});
					let action = properties.remove(&ItemProp::Action).and_then(|i| match i {
						PropValue::Vec(v) => Some(v),
						_ => None,
					});
					let dice_pool = properties
						.remove(&ItemProp::DicePool)
						.and_then(|i| match i {
							PropValue::Vec(v) => convert_dice_pool(v),
							_ => None,
						});
					let duration = properties
						.remove(&ItemProp::Duration)
						.and_then(|i| match i {
							PropValue::Vec(v) => Some(v),
							_ => None,
						});

					if cost.is_some()
						|| action.is_some() || dice_pool.is_some()
						|| duration.is_some()
					{
						Some(ActionFields {
							cost: cost.unwrap_or_default(),
							dice_pool: dice_pool.unwrap_or_default(),
							action: action.unwrap_or_default(),
							duration: duration.unwrap_or_default(),
						})
					} else {
						None
					}
				},
				notes: properties
					.remove(&ItemProp::Notes)
					.and_then(|v| match v {
						PropValue::Vec(vec) => Some(vec),
						_ => None,
					})
					.unwrap_or_default(),
			},
			PageKind::MageSpell => todo!(),
		},
	}
}
