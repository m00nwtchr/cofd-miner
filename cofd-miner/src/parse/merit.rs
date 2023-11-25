use std::str::FromStr;

use anyhow::{anyhow, Result};
use cofd_meta::PageKind;
use cofd_schema::{
	book::MeritItem,
	item::{
		merit::{Merit, MeritSubItem, MeritTag},
		ActionFields,
	},
	prelude::{BookInfo, DotRange},
	prerequisites::{Prerequisite, Prerequisites},
};
use convert_case::{Case, Casing};
use lazy_static::lazy_static;
use regex::{Captures, Regex};

use crate::source::Section;

use super::{
	get_body, get_book_reference, item::ItemProp, normalize, parse_name, process_action,
	to_paragraphs, PROP_REGEX,
};

lazy_static! {
	static ref MERIT_HEADER_REGEX: Regex = Regex::new(
		r"(?xmi)
		^
		(?P<name>[^\s.][^\n.]+)               # Name
		\s?
		\(
			(?: (?P<ltags> [^•\n]+ ) [,;] \s)?       # Tags
			(?P<cost>                         # Cost
				(?:          
					•{1,5}
					[,\s\+]*

					(?:to|or)?
					\s*
				)+
			)
			(?: [,;] \s (?P<rtags> [^•\n]+ ) )? # Tags
		\)
		( (?P<sub>:) \s (?P<subbegin> .* ) )?
		\s?
		$
	"
	)
	.unwrap();
}

pub fn parse_merits(info: &BookInfo, section: &Section) -> Result<Vec<MeritItem>> {
	let mut out = Vec::new();
	let mut children: Vec<MeritSubItem> = Vec::new();

	let mut str_pos = section.extract.len();

	let PageKind::Merit(additional_prerequisites) = &section.kind else {
		unreachable!()
	};
	let additional_prerequisites = additional_prerequisites
		.as_ref()
		.and_then(|prerqs| Prerequisites::from_str(prerqs).ok());

	for captures in MERIT_HEADER_REGEX
		.captures_iter(&section.extract)
		.collect::<Vec<_>>()
		.into_iter()
		.rev()
	{
		let sub = captures.name("sub").is_some();
		let sub_begin = captures.name("subbegin");
		let cost = captures.name("cost").unwrap();

		let name = parse_name(&captures);
		let reference = get_book_reference(&captures, section, info);
		let tags = process_tags(&captures)?;

		let body = get_body(&mut str_pos, &section.extract, &captures);
		let (mut description, mut prerequisites, effects, notes, drawbacks, action) =
			process_body(&body);
		if let Some(prereqs) = additional_prerequisites.clone() {
			if !sub {
				prerequisites.extend(prereqs.unwrap());
			}
		}
		let prerequisites = Prerequisites::from(prerequisites);
		let dot_rating = DotRange::from_str(cost.as_str())?;

		if sub {
			children.push(MeritSubItem {
				name: name.clone(),
				description: if let Some(sub_begin) = sub_begin {
					description.insert(0, normalize(sub_begin.as_str()));
					to_paragraphs(description)
				} else {
					description
				}, // TODO: ?
				prerequisites,
				dot_rating,
				drawbacks,
			});
		} else {
			children.reverse();
			out.push(MeritItem {
				name,
				reference,
				description,
				effects,
				inner: Merit {
					dot_rating,
					prerequisites,
					tags,
					drawbacks,
					children,
					action,
					notes,
				},
			});
			children = Vec::new();
		}
	}

	Ok(out)
}

fn process_body(
	body: &[String],
) -> (
	Vec<String>,
	Vec<Prerequisite>,
	Vec<String>,
	Vec<String>,
	Vec<String>,
	Option<ActionFields>,
) {
	let mut lines: Vec<String> = Vec::new();

	let mut prerequisites = Vec::new();
	let mut effects = Vec::new();
	let mut notes = Vec::new();
	let mut drawbacks = Vec::new();

	let mut action = None;

	for el in body.iter().rev() {
		if let Some(prop) = PROP_REGEX.captures(el) {
			if let (Some(prop_key), Some(prop_val)) = (prop.get(1), prop.get(2)) {
				let prop_key = ItemProp::from_str(prop_key.as_str()).unwrap();

				lines.push(prop_val.as_str().to_owned());
				lines.reverse();
				match prop_key {
					ItemProp::Prerequisites => {
						prerequisites.extend(
							to_paragraphs(lines)
								.join(" ")
								.split(", ")
								.filter_map(|str| Prerequisite::from_str(str).ok()),
						);
					}
					ItemProp::Effects => effects = to_paragraphs(lines),
					ItemProp::Notes => notes = to_paragraphs(lines),
					ItemProp::Drawbacks => drawbacks = to_paragraphs(lines),
					_ => process_action(&mut action, prop_key, lines),
				}
				lines = Vec::new();
			}
		} else {
			lines.push(el.clone());
		}
	}

	lines.reverse();
	(
		to_paragraphs(lines),
		prerequisites,
		effects,
		notes,
		drawbacks,
		action,
	)
}

fn process_tags(captures: &Captures<'_>) -> Result<Vec<MeritTag>> {
	let ltags = captures
		.name("ltags")
		.map(|m| m.as_str().to_case(Case::Title));
	let rtags = captures
		.name("rtags")
		.map(|m| m.as_str().to_case(Case::Title));

	if let Some(tags) = ltags.or(rtags) {
		let res: anyhow::Result<Vec<MeritTag>> = tags
			.split(", ")
			.map(String::from)
			.map(|s| MeritTag::from_str(&s).map_err(|err| anyhow!("{err}: {s}")))
			.collect();
		res
	} else {
		Ok(Vec::new())
	}
}

fn convert_tags(vec: Vec<String>) -> Vec<MeritTag> {
	let mut tags = Vec::new();

	for str in vec {
		if let Ok(prereq) = MeritTag::from_str(&str) {
			tags.push(prereq);
		}
	}

	tags
}