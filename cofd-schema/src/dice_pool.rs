use std::{
	ops::{Add, Sub},
	str::FromStr,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::traits::{attribute::Attribute, skill::Skill, Trait};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, derive_more::Display)]
#[serde(tag = "kind", content = "value")]
pub enum DicePool {
	Mod(i8),

	Trait(Trait),

	#[display(fmt = "Lower of {_0} and {_1}")]
	Min(Box<DicePool>, Box<DicePool>),
	#[display(fmt = "Higher of {_0} and {_1}")]
	Max(Box<DicePool>, Box<DicePool>),

	#[display(fmt = "{}", "_0.iter().join(\" + \")")]
	Add(Vec<DicePool>),
	#[display(fmt = "{_0} - {_1}")]
	Sub(Box<DicePool>, Box<DicePool>),
	#[display(fmt = "{_0} vs {_1}")]
	Vs(Box<DicePool>, Box<DicePool>),

	Key(String),
}

impl DicePool {
	// pub fn value(&self, character: &Character) -> i16 {
	// 	match self {
	// 		Self::Mod(val) => *val,
	// 		Self::Attribute(attr) => *character.attributes().get(attr) as i16,
	// 		Self::Skill(skill) => character.skills().get(*skill) as i16,
	// 		Self::Trait(trait_) => character.get_trait(trait_) as i16,
	// 		Self::Add(p1, p2) => p1.value(character) + p2.value(character),
	// 		Self::Sub(p1, p2) => p1.value(character) - p2.value(character),
	// 		Self::Max(p1, p2) => max(p1.value(character), p2.value(character)),
	// 		Self::Min(p1, p2) => min(p1.value(character), p2.value(character)),
	// 	}
	// }

	pub fn min(p1: impl Into<DicePool>, p2: impl Into<DicePool>) -> DicePool {
		DicePool::Min(Box::new(p1.into()), Box::new(p2.into()))
	}

	pub fn max(p1: impl Into<DicePool>, p2: impl Into<DicePool>) -> DicePool {
		DicePool::Max(Box::new(p1.into()), Box::new(p2.into()))
	}
}

impl FromStr for DicePool {
	type Err = strum::ParseError;

	fn from_str(str: &str) -> Result<Self, Self::Err> {
		let mut char = None;

		if let Some((l, r)) = str.split_once("vs") {
			let p1 = DicePool::from_str(l.trim())?;
			let p2 = DicePool::from_str(r.trim_matches(&['.', ' '][..]))?;

			Ok(DicePool::Vs(Box::new(p1), Box::new(p2)))
		} else if let Some((l, r)) = str.split_once("versus") {
			let p1 = DicePool::from_str(l.trim())?;
			let p2 = DicePool::from_str(r.trim_matches(&['.', ' '][..]))?;

			Ok(DicePool::Vs(Box::new(p1), Box::new(p2)))
		} else if let Some((l, r)) = str.rsplit_once(|c: char| {
			let f = c.eq(&'+') || c.eq(&'-');
			if f {
				char = Some(c);
			}
			f
		}) {
			let p1 = DicePool::from_str(l.trim())?;
			let p2 = DicePool::from_str(r.trim())?;

			Ok(match char {
				Some('+') => p1 + p2,
				Some('-') => p1 - p2,
				_ => unreachable!(),
			})
		} else {
			Trait::from_str(str)
				.map(DicePool::Trait)
				.or_else(|_| Ok(DicePool::Key(str.to_owned())))
		}
	}
}

impl Default for DicePool {
	fn default() -> Self {
		Self::Mod(0)
	}
}

impl<T: Into<DicePool>> Add<T> for DicePool {
	type Output = DicePool;

	fn add(self, rhs: T) -> Self::Output {
		let rhs = rhs.into();
		let mut vec = match self {
			DicePool::Add(v) => v,
			_ => vec![self],
		};

		match rhs {
			DicePool::Add(v) => vec.extend(v),
			_ => vec.push(rhs),
		}

		Self::Add(vec)
	}
}

impl<T: Into<DicePool>> Sub<T> for DicePool {
	type Output = DicePool;

	fn sub(self, rhs: T) -> Self::Output {
		Self::Sub(Box::new(self), Box::new(rhs.into()))
	}
}

impl From<i8> for DicePool {
	fn from(val: i8) -> Self {
		Self::Mod(val)
	}
}

impl<T: Into<Trait>> From<T> for DicePool {
	fn from(value: T) -> Self {
		DicePool::Trait(value.into())
	}
}

impl<T: Into<DicePool>> Add<T> for Attribute {
	type Output = DicePool;

	fn add(self, rhs: T) -> Self::Output {
		DicePool::Trait(self.into()) + rhs.into()
	}
}

impl<T: Into<DicePool>> Add<T> for Skill {
	type Output = DicePool;

	fn add(self, rhs: T) -> Self::Output {
		DicePool::Trait(self.into()) + rhs.into()
	}
}

impl From<Vec<DicePool>> for DicePool {
	fn from(mut value: Vec<DicePool>) -> Self {
		if value.len() > 1 {
			Self::Add(value)
		} else if let Some(value) = value.pop() {
			value
		} else {
			Self::Mod(0)
		}
	}
}
