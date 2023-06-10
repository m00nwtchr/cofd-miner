use super::traits::{Attribute, Skill, Template};

pub enum Prerequisite {
	Template(Template),
	Attribute(Attribute),
	Skill(Skill),
	Any(PrereqKind),
}

pub enum PrereqKind {
	Attribute,
	Skill,
}

pub struct Prerequisites {}
