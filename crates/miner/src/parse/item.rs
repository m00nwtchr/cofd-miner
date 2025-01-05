use std::{collections::HashMap, str::FromStr};

use anyhow::anyhow;
use cofd_schema::{
	dice_pool::DicePool,
	prerequisites::{Prerequisite, Prerequisites},
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum::EnumString;

use crate::parse::{paragraph::to_paragraphs, process_action};

pub static PROP_REGEX: Lazy<Regex> = Lazy::new(|| {
	Regex::new(
		r"^(Prerequisite|Style Tag|Cost|Dice Pool|Action|Duration|Effect|Drawback|Note|Exceptional Success|Success|Failure|Dramatic Failure|Suggested Modifiers)s?:\s?(.*)$"
	)
		.unwrap()
});

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, Hash, PartialEq, Eq)]
#[strum(ascii_case_insensitive)]
#[serde(rename_all = "camelCase")]
pub enum ItemProp {
	#[strum(serialize = "Prerequisites", serialize = "Prerequisite")]
	Prerequisites,
	#[strum(to_string = "Style Tags", serialize = "Style Tag")]
	StyleTags,

	Cost,
	#[strum(to_string = "Dice Pool")]
	DicePool,
	Action,
	Duration,
	#[strum(to_string = "Exceptional Success")]
	ExceptionalSuccess,
	Success,
	Failure,
	#[strum(to_string = "Dramatic Failure")]
	DramaticFailure,
	#[strum(to_string = "Suggested Modifiers")]
	SuggestedModifiers,

	#[strum(serialize = "Effects", serialize = "Effect")]
	Effects,
	#[strum(serialize = "Drawbacks", serialize = "Drawback")]
	Drawbacks,
	#[strum(serialize = "Notes", serialize = "Note")]
	Notes,
}

#[derive(Debug, Default)]
pub struct RawItem(HashMap<Option<ItemProp>, Vec<String>>);
impl RawItem {
	pub fn push(&mut self, prop: Option<ItemProp>, value: Vec<String>) {
		self.0.entry(prop).or_default().extend(value);
	}

	pub fn get(&self, prop: Option<ItemProp>) -> &Vec<String> {
		static EMPTY_VEC: Vec<String> = Vec::new();

		self.0.get(&prop).unwrap_or(&EMPTY_VEC)
	}

	pub fn take(&mut self, prop: Option<ItemProp>) -> Vec<String> {
		self.0.remove(&prop).unwrap_or_default()
	}

	pub fn get_mut(&mut self, prop: Option<ItemProp>) -> Option<&mut Vec<String>> {
		self.0.get_mut(&prop)
	}

	pub fn has_properties(&self) -> bool {
		self.0.keys().any(|p| p.is_some())
	}
}

impl TryFrom<Vec<String>> for RawItem {
	type Error = anyhow::Error;

	fn try_from(body: Vec<String>) -> Result<Self, Self::Error> {
		let mut lines: Vec<String> = Vec::new();
		// let mut first_prop = true;

		let mut raw_item = RawItem::default();

		for line in body.iter().rev() {
			if let Some(prop) = PROP_REGEX.captures(line.trim_start()) {
				if let (Some(prop_key), Some(prop_val)) = (prop.get(1), prop.get(2)) {
					let prop_key = ItemProp::from_str(prop_key.as_str())?;
					let line = prop_val.as_str();

					lines.push(line.to_owned());
					// Effects get reversed later
					if prop_key != ItemProp::Effects {
						lines.reverse();
					}
					match prop_key {
						ItemProp::Prerequisites => raw_item.push(
							Some(ItemProp::Prerequisites),
							to_paragraphs(&lines)[0]
								.split(", ")
								.map(|s| s.to_string())
								.collect(),
						),
						ItemProp::Effects => raw_item.push(Some(ItemProp::Effects), lines), // Effects are rolled into paragraphs later
						_ => raw_item.push(Some(prop_key), to_paragraphs(&lines)),
					}
					lines = Vec::new();
				}
			} else if line.starts_with('\t') {
				lines.push(line.to_owned());
				raw_item.push(
					if raw_item.has_properties() {
						None
					} else {
						Some(ItemProp::Effects)
					},
					lines,
				);
				lines = Vec::new();
			} else {
				lines.push(line.to_owned());
			}
		}

		if !lines.is_empty() {
			raw_item.push(None, lines);
		}

		// Allow descriptions to be rolled into paragraphs later
		raw_item.get_mut(None).iter_mut().for_each(|d| d.reverse());
		raw_item
			.get_mut(Some(ItemProp::Effects))
			.iter_mut()
			.for_each(|d| {
				d.reverse();
				**d = to_paragraphs(d);
			});

		Ok(raw_item)
	}
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

pub fn convert_dice_pool(vec: &[String]) -> Option<DicePool> {
	if vec.len() == 1 {
		let str = vec.first().unwrap();

		if let Ok(pool) = DicePool::from_str(str) {
			return Some(pool);
		}
	}
	None
}
