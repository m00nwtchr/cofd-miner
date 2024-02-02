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

use crate::parse::paragraph::to_paragraphs;
use crate::source::Section;

use super::{
	get_body, get_book_reference, item::ItemProp, normalize, parse_name, process_action, PROP_REGEX,
};

lazy_static! {
	static ref MERIT_HEADER_REGEX: Regex = Regex::new(
		r"(?xmi)
		^\t*
		(?<name> (?:\w{2,3}\.\s)? (?:\t?[\w\-\']\s*)+)  # Name
		\s?
		\(
			(?: (?<ltags> [^•\n]+ ) [,;] \s)?           # Tags
			(?<cost>                                    # Cost
				(?:          
					•{1,5}
					[,\s\+]*

					(?:to|or)?
					\s*
				)+
			)
			(?: [,;] \s (?<rtags> [^•\n]+ ) )?          # Tags
		\)
		(?: : \s (?<sub> .* ) )?
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
		let sub = captures.name("sub");
		let cost = captures.name("cost").unwrap();

		let name = parse_name(&captures);
		let reference = get_book_reference(&captures, section, info);
		let tags = process_tags(&captures)?;

		let body = get_body(&mut str_pos, &section.extract, &captures);
		let mut body = parse_body(&body);

		if let Some(prereqs) = additional_prerequisites.clone() {
			if sub.is_none() {
				body.prerequisites.extend(prereqs.unwrap());
			}
		}
		let prerequisites = Prerequisites::from(body.prerequisites);
		let dot_rating = DotRange::from_str(cost.as_str())?;

		if let Some(sub) = sub {
			body.description.insert(0, {
				let mut desc = normalize(sub.as_str());
				desc.insert(0, '\t');
				desc
			});
			children.push(MeritSubItem {
				name: name.clone(),
				description: to_paragraphs(body.description),
				prerequisites,
				dot_rating,
				drawbacks: body.drawbacks,
			});
		} else {
			children.reverse();
			out.push(MeritItem {
				name,
				reference,
				description: to_paragraphs(body.description),
				effects: body.effects,
				inner: Merit {
					dot_rating,
					prerequisites,
					tags,
					drawbacks: body.drawbacks,
					children,
					action: body.action,
					notes: body.notes,
				},
			});
			children = Vec::new();
		}
	}

	Ok(out)
}

/**
 * Structure which holds the results of [`parse_body`]
 */
pub struct MeritBody {
	pub description: Vec<String>,
	pub prerequisites: Vec<Prerequisite>,
	pub effects: Vec<String>,
	pub notes: Vec<String>,
	pub drawbacks: Vec<String>,
	pub action: Option<ActionFields>,
}

fn parse_body(body: &[String]) -> MeritBody {
	let mut lines: Vec<String> = Vec::new();

	let mut description = Vec::new();
	let mut prerequisites = Vec::new();
	let mut effects = Vec::new();
	let mut notes = Vec::new();
	let mut drawbacks = Vec::new();
	let mut action = None;

	let mut first_prop = true;

	for line in body.iter().rev() {
		if let Some(prop) = PROP_REGEX.captures(line.trim_start()) {
			if let (Some(prop_key), Some(prop_val)) = (prop.get(1), prop.get(2)) {
				let prop_key = ItemProp::from_str(prop_key.as_str()).unwrap();
				let line = prop_val.as_str();

				if !effects.is_empty() {
					first_prop = false;
				}

				lines.push(line.to_owned());
				// Effects get reversed later
				if prop_key != ItemProp::Effects {
					lines.reverse();
				}
				match prop_key {
					ItemProp::Prerequisites => {
						prerequisites.extend(
							to_paragraphs(lines)[0]
								.split(", ")
								.filter_map(|str| Prerequisite::from_str(str).ok()),
						);
					}
					ItemProp::Effects => effects.extend(lines), // Effects are rolled into paragraphs later
					ItemProp::Notes => notes.extend(to_paragraphs(lines)),
					ItemProp::Drawbacks => drawbacks.extend(to_paragraphs(lines)),
					_ => process_action(&mut action, prop_key, to_paragraphs(lines)),
				}
				lines = Vec::new();
			}
		} else if line.starts_with('\t') {
			lines.push(line.to_owned());
			if first_prop {
				effects.extend(lines);
			} else {
				description.extend(lines);
			}
			lines = Vec::new();
		} else {
			lines.push(line.to_owned());
		}
	}

	if !lines.is_empty() {
		description.extend(lines);
	}

	description.reverse();
	effects.reverse();
	// description = to_paragraphs(description); // Allow descriptions to be rolled into paragraphs later
	effects = to_paragraphs(effects);

	MeritBody {
		description,
		prerequisites,
		effects,
		notes,
		drawbacks,
		action,
	}
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
