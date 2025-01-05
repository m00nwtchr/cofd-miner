use std::{collections::HashMap, str::FromStr};

use cofd_schema::{
	dice_pool::DicePool,
	item::{ActionFields, RollResults},
	modifiers::SuggestedModifiers,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum::EnumString;

use crate::parse::paragraph::to_paragraphs;

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

	// pub fn has_properties(&self) -> bool {
	// 	self.0
	// 		.keys()
	// 		.any(|p| !(p.is_none() || p.eq(&Some(ItemProp::Effects))))
	// }

	pub fn action(&mut self) -> Option<ActionFields> {
		if self.0.keys().any(|k| {
			matches!(
				k,
				Some(
					ItemProp::Action
						| ItemProp::Cost | ItemProp::DicePool
						| ItemProp::Duration
						| ItemProp::Success
						| ItemProp::Failure
						| ItemProp::DramaticFailure
						| ItemProp::ExceptionalSuccess
						| ItemProp::SuggestedModifiers
				)
			)
		}) {
			Some(ActionFields {
				action: self.take(Some(ItemProp::Action)),
				cost: self.take(Some(ItemProp::Cost)),
				dice_pool: convert_dice_pool(&self.take(Some(ItemProp::DicePool))),
				duration: self.take(Some(ItemProp::Duration)),
				roll_results: RollResults {
					exceptional_success: self.take(Some(ItemProp::ExceptionalSuccess)),
					success: self.take(Some(ItemProp::Success)),
					failure: self.take(Some(ItemProp::Failure)),
					dramatic_failure: self.take(Some(ItemProp::DramaticFailure)),
				},
				suggested_modifiers: SuggestedModifiers::from_str(
					&self.take(Some(ItemProp::SuggestedModifiers)).concat(),
				)
				.unwrap_or_default(),
			})
		} else {
			None
		}
	}
}

impl TryFrom<Vec<&str>> for RawItem {
	type Error = anyhow::Error;

	fn try_from(body: Vec<&str>) -> Result<Self, Self::Error> {
		let mut lines: Vec<String> = Vec::new();
		let mut first_prop = true;

		let mut raw_item = RawItem::default();

		for line in body.into_iter().rev() {
			if let Some(prop) = PROP_REGEX.captures(line.trim_start()) {
				if let (Some(prop_key), Some(prop_val)) = (prop.get(1), prop.get(2)) {
					let prop_key = ItemProp::from_str(prop_key.as_str())?;
					let line = prop_val.as_str();

					if !raw_item.get(Some(ItemProp::Effects)).is_empty() {
						first_prop = false;
					}

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
								.map(ToString::to_string)
								.collect(),
						),
						ItemProp::Effects => raw_item.push(Some(ItemProp::Effects), lines), // Effects are rolled into paragraphs later
						_ => raw_item.push(Some(prop_key), to_paragraphs(&lines)),
					}
					lines = Vec::new();
				}
			} else if line.starts_with('\t') {
				lines.push(line.to_string());
				raw_item.push(
					if first_prop {
						Some(ItemProp::Effects)
					} else {
						None
					},
					lines,
				);
				lines = Vec::new();
			} else {
				lines.push(line.to_string());
			}
		}

		if !lines.is_empty() {
			raw_item.push(None, lines);
		}

		// Allow descriptions to be rolled into paragraphs later
		if let Some(descriptions) = raw_item.get_mut(None) {
			descriptions.reverse();
		}
		if let Some(effects) = raw_item.get_mut(Some(ItemProp::Effects)) {
			effects.reverse();
			*effects = to_paragraphs(effects);
		}

		Ok(raw_item)
	}
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
