mod merit;
// mod prerequisites;
mod dot_range;
mod traits;

pub mod prelude {
	pub use super::dot_range::DotRange;
	pub use super::merit::{Merit, MeritFeature};
	pub use super::traits::{Attribute, Skill};
}
