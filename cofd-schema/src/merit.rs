use serde::{Deserialize, Serialize};

use super::prelude::DotRange;

#[derive(Serialize, Deserialize)]
pub struct MeritFeature {
	pub name: String,
	pub description: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Merit {
	pub name: String,
	pub rating: DotRange,
	pub description: Vec<String>,
	pub features: Vec<MeritFeature>,
}
