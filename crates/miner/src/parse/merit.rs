use std::str::FromStr;

use anyhow::{anyhow, Result};
use cofd_meta::PageKind;
use cofd_schema::{
	book::MeritItem,
	item::merit::{Merit, MeritSubItem, MeritTag},
	prelude::{BookInfo, DotRange},
	prerequisites::{Prerequisite, Prerequisites},
};
use convert_case::{Case, Casing};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};

use super::{get_body, get_book_reference, item::ItemProp, normalize, parse_name};
use crate::{
	parse::{item::RawItem, paragraph::to_paragraphs},
	source::Section,
};

static MERIT_HEADER_REGEX: Lazy<Regex> = Lazy::new(|| {
	Regex::new(
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
	",
	)
	.unwrap()
});

pub fn parse_merits(info: &BookInfo, section: &Section) -> Result<Vec<MeritItem>> {
	let mut out = Vec::new();
	let mut children: Vec<MeritSubItem> = Vec::new();

	let mut str_pos = section.extract.len();

	let PageKind::Merit(additional_prerequisites) = &section.kind else {
		unreachable!()
	};
	let additional_prerequisites = additional_prerequisites
		.as_ref()
		.and_then(|prereqs| Prerequisites::from_str(prereqs).ok());

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
		let mut raw_item = {
			let v: Vec<&str> = body.iter().map(String::as_str).collect();
			RawItem::try_from(v)?
		};

		let mut prerequisites: Vec<Prerequisite> = raw_item
			.get(Some(ItemProp::Prerequisites))
			.iter()
			.filter_map(|p| Prerequisite::from_str(p).ok())
			.collect();
		if sub.is_none() {
			if let Some(prereqs) = &additional_prerequisites {
				prerequisites.extend(prereqs.clone().unwrap());
			}
		}

		let prerequisites = Prerequisites::from(prerequisites);
		let dot_rating = DotRange::from_str(cost.as_str())?;

		if let Some(sub) = sub {
			let mut description = raw_item.take(None);

			description.insert(0, {
				let mut desc = normalize(sub.as_str());
				desc.insert(0, '\t');
				desc
			});
			children.push(MeritSubItem {
				name: name.clone(),
				description: to_paragraphs(&description),
				prerequisites,
				dot_rating,
				drawbacks: raw_item.take(Some(ItemProp::Drawbacks)),
			});
		} else {
			children.reverse();
			out.push(MeritItem {
				name,
				reference,
				description: to_paragraphs(raw_item.get(None)),
				effects: raw_item.take(Some(ItemProp::Effects)),
				inner: Merit {
					dot_rating,
					prerequisites,
					tags,
					drawbacks: raw_item.take(Some(ItemProp::Drawbacks)),
					children,
					action: raw_item.action(),
					notes: raw_item.take(Some(ItemProp::Notes)),
				},
			});
			children = Vec::new();
		}
	}

	Ok(out)
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
