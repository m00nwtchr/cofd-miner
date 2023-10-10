use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::item::{ItemProp, PropValue};
use crate::prelude::DotRange;

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct MeritFeature {
	pub name: String,
	pub description: Vec<String>,
	pub effects: Vec<String>,
	pub extras: Option<HashMap<ItemProp, PropValue>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
pub struct Merit {
	pub name: String,
	pub rating: DotRange,
	pub description: Vec<String>,
	pub features: Vec<MeritFeature>,
	pub extras: Option<HashMap<ItemProp, PropValue>>,
}
