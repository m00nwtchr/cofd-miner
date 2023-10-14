use std::convert::AsRef;
use std::{
	fmt::Display,
	ops::{Add, Sub},
};

use serde::{Deserialize, Serialize};

use crate::traits::attribute::Attribute;
use crate::traits::skill::Skill;
use crate::traits::Trait;

#[derive(Debug, Serialize, Deserialize)]
pub enum DicePool {
	Mod(i16),
	Attribute(Attribute),
	Skill(Skill),
	Trait(Trait),

	Min(Box<DicePool>, Box<DicePool>),
	Max(Box<DicePool>, Box<DicePool>),

	Add(Box<DicePool>, Box<DicePool>),
	Sub(Box<DicePool>, Box<DicePool>),
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

impl Default for DicePool {
	fn default() -> Self {
		Self::Mod(0)
	}
}

impl<T: Into<DicePool>> Add<T> for DicePool {
	type Output = DicePool;

	fn add(self, rhs: T) -> Self::Output {
		Self::Add(Box::new(self), Box::new(rhs.into()))
	}
}

impl<T: Into<DicePool>> Sub<T> for DicePool {
	type Output = DicePool;

	fn sub(self, rhs: T) -> Self::Output {
		Self::Sub(Box::new(self), Box::new(rhs.into()))
	}
}

impl<T: Into<Attribute>> From<T> for DicePool {
	fn from(value: T) -> Self {
		DicePool::Attribute(value.into())
	}
}

impl From<Skill> for DicePool {
	fn from(skill: Skill) -> Self {
		Self::Skill(skill)
	}
}

impl From<Trait> for DicePool {
	fn from(trait_: Trait) -> Self {
		Self::Trait(trait_)
	}
}

impl From<i16> for DicePool {
	fn from(val: i16) -> Self {
		Self::Mod(val)
	}
}

impl Add for Attribute {
	type Output = DicePool;

	fn add(self, rhs: Self) -> Self::Output {
		DicePool::Attribute(self) + DicePool::Attribute(rhs)
	}
}

impl Add<Skill> for Attribute {
	type Output = DicePool;

	fn add(self, rhs: Skill) -> Self::Output {
		DicePool::Attribute(self) + DicePool::Skill(rhs)
	}
}

impl Display for DicePool {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			DicePool::Mod(val) => val.fmt(f),
			DicePool::Attribute(attr) => f.write_str(attr.as_ref()),
			DicePool::Skill(skill) => f.write_str(skill.as_ref()),
			DicePool::Trait(trait_) => f.write_str(trait_.as_ref()),
			DicePool::Min(p1, p2) => f.write_fmt(format_args!("min({p1}, {p2})")),
			DicePool::Max(p1, p2) => f.write_fmt(format_args!("max({p1}, {p2})")),
			DicePool::Add(p1, p2) => f.write_fmt(format_args!("{p1} + {p2}")),
			DicePool::Sub(p1, p2) => f.write_fmt(format_args!("{p1} - {p2}")),
		}
	}
}
