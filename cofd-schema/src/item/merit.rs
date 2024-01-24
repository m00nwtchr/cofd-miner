use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

use super::ActionFields;
use crate::{prelude::DotRange, prerequisites::Prerequisites};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, AsRefStr, PartialEq, Eq)]
#[strum(ascii_case_insensitive)]
pub enum MeritTag {
	Style,
	#[strum(to_string = "Supernatural Merit")]
	Supernatural,

	Special,

	// TODO: pdf parse
	Mental,
	Social,
	Physical,

	Fighting,
}

impl Display for MeritTag {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_ref())
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MeritSubItem {
	pub name: String,
	pub description: Vec<String>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub prerequisites: Prerequisites,
	pub dot_rating: DotRange,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub drawbacks: Vec<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Merit {
	pub dot_rating: DotRange,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub prerequisites: Prerequisites,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub tags: Vec<MeritTag>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub drawbacks: Vec<String>,

	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub children: Vec<MeritSubItem>,

	// #[serde(default, skip_serializing_if = "Option::is_none")]
	#[serde(flatten)]
	pub action: Option<ActionFields>,

	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub notes: Vec<String>,
}
