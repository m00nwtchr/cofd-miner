use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::{
	error::{self, ParseError},
	item::{
		gift::{Gift, Moon, Other},
		merit::Merit,
		spell::Spell,
		Item,
	},
};

#[derive(
	Default, Debug, Clone, Copy, Serialize, Deserialize, EnumString, Display, PartialEq, Eq,
)]
#[strum(ascii_case_insensitive)]
pub enum BookId {
	CofD,
	HL,
	DE,
	DE2,

	#[strum(to_string = "MtA 2e", serialize = "MtA2e")]
	MtA2e,
	SoS,

	#[strum(to_string = "VtR 2e", serialize = "VtR2e")]
	VtR2e,

	#[strum(to_string = "WtF 2e", serialize = "WtF2e")]
	WtF2e,
	#[strum(to_string = "NH-SM", serialize = "NHSM")]
	NHSM,

	#[strum(to_string = "CtL 2e", serialize = "CtL2e")]
	CtL2e,

	#[strum(to_string = "MtC 2e", serialize = "MtC2e")]
	MtC2e,

	DtD,

	BtP,

	DtR,

	#[default]
	Codex,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BookInfo {
	pub name: String,
	pub id: BookId,
	#[serde(with = "hex")]
	pub hash: u64,
	pub publication_date: chrono::NaiveDate,
}

impl BookInfo {
	#[must_use]
	pub fn new(name: String, id: BookId) -> Self {
		BookInfo {
			name,
			id,
			hash: 0,
			publication_date: chrono::NaiveDate::default(),
		}
	}
}

pub type MeritItem = Item<Merit>;
pub type SpellItem = Item<Spell>;
pub type MoonGift = Gift<Moon>;
pub type OtherGift = Gift<Other>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Book {
	pub info: BookInfo,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub merits: Vec<MeritItem>,

	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub mage_spells: Vec<SpellItem>,

	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub moon_gifts: Vec<MoonGift>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub gifts: Vec<OtherGift>,
}

impl From<BookInfo> for Book {
	fn from(info: BookInfo) -> Self {
		Book {
			info,
			merits: Vec::new(),
			mage_spells: Vec::new(),
			moon_gifts: Vec::new(),
			gifts: Vec::new(),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, derive_more::Display)]
#[display(fmt = "{_0} pg.{_1}")]
pub struct BookReference(pub BookId, pub usize);

impl Default for BookReference {
	fn default() -> Self {
		Self(BookId::CofD, Default::default())
	}
}

impl FromStr for BookReference {
	type Err = error::ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut split = s
			.split(|c: char| c.is_whitespace() || c.eq(&'p'))
			.filter(|s| !s.is_empty());

		let first = split.next();
		let second = split.next();
		let last = split.last();

		if let (Some(first), Some(second), Some(last)) = (first, second, last) {
			let last = last.trim();

			Ok(BookReference(
				BookId::from_str(&(first.trim().to_owned() + second.trim()))
					.map_err(ParseError::from)?,
				usize::from_str(if let Some((l, _)) = last.rsplit_once('-') {
					l
				} else {
					last
				})
				.map_err(ParseError::from)?,
			))
		} else if let (Some(first), Some(second)) = (first, second) {
			let second = second.trim();

			Ok(BookReference(
				BookId::from_str(first.trim()).map_err(ParseError::from)?,
				usize::from_str(if let Some((l, _)) = second.rsplit_once('-') {
					l
				} else {
					second
				})
				.map_err(ParseError::from)?,
			))
		} else {
			Err(error::ParseError::BadFormat(s.to_owned()))
		}
	}
}

mod hex {
	use serde::{Deserialize, Serialize};

	#[allow(clippy::trivially_copy_pass_by_ref)]
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
			String::deserialize(deserializer).map(|str| u64::from_str_radix(&str, 16).unwrap())
		} else {
			u64::deserialize(deserializer)
		}
	}
}
