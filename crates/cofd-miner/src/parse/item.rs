use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::EnumString;

use cofd_schema::{
	dice_pool::DicePool,
	prerequisites::{Prerequisite, Prerequisites},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, PartialEq, Eq)]
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
