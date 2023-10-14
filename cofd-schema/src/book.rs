use serde::{Deserialize, Serialize};

use crate::item::Item;

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BookInfo {
	#[serde(with = "hex")]
	pub hash: u64,
	pub publication_date: chrono::NaiveDate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Book {
	pub info: BookInfo,
	#[serde(default, skip_serializing_if = "crate::is_empty")]
	pub merits: Vec<Item>,
	#[serde(default, skip_serializing_if = "crate::is_empty")]
	pub mage_spells: Vec<Item>,
}

mod hex {
	use serde::{Deserialize, Serialize};

	pub fn serialize<S>(v: &u64, serializer: S) -> std::result::Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		if serializer.is_human_readable() {
			format!("{v:X}").serialize(serializer)
		} else {
			v.serialize(serializer)
		}
	}
	pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<u64, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		if deserializer.is_human_readable() {
			String::deserialize(deserializer)
				.and_then(|str| Ok(u64::from_str_radix(&str, 16).unwrap()))
		} else {
			u64::deserialize(deserializer)
		}
	}
}
