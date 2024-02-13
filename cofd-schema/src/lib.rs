#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod book;
pub mod dice_pool;
pub mod dot_range;
pub mod error;
pub mod item;
pub mod modifiers;
pub mod prerequisites;
pub mod splat;
pub mod traits;

pub mod prelude {
	pub use super::book::BookInfo;
	pub use super::dot_range::DotRange;
	pub use super::traits::attribute::{
		Attribute, MentalAttribute, PhysicalAttribute, SocialAttribute,
	};
	pub use super::traits::skill::{MentalSkill, PhysicalSkill, Skill, SocialSkill};
}

pub static DOT_CHAR: char = '•';

trait ByName {}
