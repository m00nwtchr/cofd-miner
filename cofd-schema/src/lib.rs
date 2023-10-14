use std::collections::HashMap;

pub mod book;
pub mod dice_pool;
pub mod dot_range;
pub mod item;
pub mod prerequisites;
pub mod traits;

pub mod prelude {
	pub use super::book::BookInfo;
	pub use super::dot_range::DotRange;
	pub use super::item::merit::{Merit, MeritFeature};
	pub use super::traits::attribute::{
		Attribute, MentalAttribute, PhysicalAttribute, SocialAttribute,
	};
	pub use super::traits::skill::{MentalSkill, PhysicalSkill, Skill, SocialSkill};
}

pub static DOT_CHAR: char = '•';

pub(crate) fn is_empty_map<K, V>(map: &HashMap<K, V>) -> bool {
	map.is_empty()
}

pub(crate) fn is_empty<T>(vec: &Vec<T>) -> bool {
	vec.is_empty()
}

trait ByName {}
