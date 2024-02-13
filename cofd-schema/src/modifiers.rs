use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct SuggestedModifiers(Vec<(String, i8)>);

impl FromStr for SuggestedModifiers {
	type Err = ();

	fn from_str(str: &str) -> Result<Self, Self::Err> {
		Ok(SuggestedModifiers(
			str.split(", ")
				.filter_map(|s| {
					s.rsplit_once(" (")
						.map(|(l, r)| (l, r.trim_end_matches(')')))
				})
				.filter_map(|(l, r)| r.parse().map(|r| (l.to_string(), r)).ok())
				.collect(),
		))
	}
}

impl SuggestedModifiers {
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}
