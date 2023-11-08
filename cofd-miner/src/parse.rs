use std::{collections::BTreeMap, str::FromStr};

use anyhow::Result;
use convert_case::{Case, Casing};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

use crate::{
	parser_item::{convert_item, ItemProp, ParserItem, ParserSubItem, PropValue},
	source::Section,
};
use cofd_meta::PageKind;
use cofd_schema::{
	book::{Book, BookInfo, BookReference},
	item::{
		gift::{FacetKind, GiftKind},
		ItemKind,
	},
	prelude::DotRange,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfExtract {
	pub info: BookInfo,
	pub sections: Vec<Section>,
	// errors: Vec<Error>,
}

impl PdfExtract {
	pub fn parse(self) -> Result<Book> {
		let sections: Result<Vec<(PageKind, Vec<ItemKind>)>> = self
			.sections
			.par_iter()
			.map(|span| parse_span(&self.info, span).map(|parsed| (span.kind.clone(), parsed)))
			.map(|result| {
				let (kind, parsed) = result?;
				let parsed: Vec<ItemKind> = parsed
					.into_par_iter()
					.map(|item| convert_item(&kind, item))
					.collect();
				Ok((kind, parsed))
			})
			.collect();

		let mut parse = Book::from(self.info);

		for (kind, vec) in sections? {
			match kind {
				PageKind::Merit(_) => parse.merits.extend(vec.into_iter().map(|i| match i {
					ItemKind::Merit(merit) => merit,
					_ => unreachable!(),
				})),
				PageKind::MageSpell => parse.mage_spells.extend(vec.into_iter().map(|i| match i {
					_ => unreachable!(),
				})),
				PageKind::Gift(kind) => match kind {
					GiftKind::Moon => parse.moon_facets.extend(vec.into_iter().map(|i| match i {
						ItemKind::Facet(FacetKind::Moon(item)) => item,
						_ => unreachable!(),
					})),
					GiftKind::Other => parse.facets.extend(vec.into_iter().map(|i| match i {
						ItemKind::Facet(FacetKind::Other(item)) => item,
						_ => unreachable!(),
					})),
				},
			}
		}
		parse.merits.sort_by(|a, b| a.name.cmp(&b.name));
		parse.mage_spells.sort_by(|a, b| a.name.cmp(&b.name));

		Ok(parse)
	}
}

lazy_static! {
	static ref MERIT_HEADER_REGEX: Regex = Regex::new(r"(?xmi)
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
	").unwrap();

	static ref PROP_REGEX: Regex = Regex::new(r"^(Prerequisite|Style Tag|Cost|Dice Pool|Action|Duration|Effect|Drawback|Note)s?:\s?(.*)$").unwrap();
	//
}

// TODO: Some edge cases don't get merged properly but works ok overall.
fn to_paragraphs(vec: Vec<String>) -> Vec<String> {
	let mut out = Vec::new();
	let mut paragraph = String::new();

	let mut flag = false;
	for line in vec {
		if !paragraph.is_empty() && !flag {
			paragraph.push(' ');
		} else if flag {
			flag = false;
		}

		paragraph.push_str(if line.ends_with("God-") {
			flag = true;
			&line
		} else if line.ends_with('-') {
			flag = true;
			line.trim_end_matches('-')
		} else {
			&line
		});

		if line.ends_with('.') {
			out.push(paragraph);
			paragraph = String::new();
		}
	}
	if !paragraph.is_empty() {
		out.push(paragraph);
	}

	out
}

fn get_body(str_pos: &mut usize, span: &str, captures: &Captures<'_>) -> Vec<String> {
	let body = span[captures.get(0).unwrap().end()..*str_pos]
		.split('\n')
		.filter_map(filter_normalize)
		.collect();
	*str_pos = captures.get(0).unwrap().start();

	body
}

#[allow(clippy::too_many_lines)]
pub fn parse_span(info: &BookInfo, span: &Section) -> Result<Vec<ParserItem>> {
	let mut out = Vec::new();
	let mut str_pos = span.extract.len();

	let mut children: Vec<ParserSubItem> = Vec::new();

	let matches: Vec<_> = match span.kind {
		PageKind::Merit(_) => MERIT_HEADER_REGEX.captures_iter(&span.extract).collect(),
		PageKind::MageSpell => vec![],
		PageKind::Gift(_) => todo!(),
	};
	for captures in matches.into_iter().rev() {
		let mut props = BTreeMap::new();

		let name = normalize(captures.name("name").map_or("", |f| f.as_str()));
		let name = if name.chars().all(|f| f.is_uppercase() || !f.is_alphabetic()) {
			let is: Vec<_> = name
				.chars()
				.enumerate()
				.filter(|(_, c)| c.eq(&'-'))
				.map(|(i, _)| i)
				.collect();

			if is.is_empty() {
				name.to_case(Case::Title)
			} else {
				let mut name = name.to_case(Case::Title);
				for i in is {
					name.replace_range(i..=i, "-");
				}
				name
			}
		} else {
			name.to_string()
		};

		let desc = match &span.kind {
			PageKind::Merit(additional_prereqs) => {
				let cost = captures.name("cost");

				let sub = captures.name("sub").is_some();
				let sub_begin = captures.name("subbegin");

				let ltags = captures
					.name("ltags")
					.map(|m| m.as_str().to_case(Case::Title));
				let rtags = captures
					.name("rtags")
					.map(|m| m.as_str().to_case(Case::Title));

				if let Some(tags) = ltags.or(rtags) {
					props.insert(
						ItemProp::Tags,
						PropValue::Vec(tags.split(", ").map(String::from).collect()),
					);
				}

				if let Some(cost) = cost {
					props.insert(
						ItemProp::DotRating,
						PropValue::DotRange(DotRange::from_str(cost.as_str())?),
					);
				}

				if let Some(prereqs) = additional_prereqs {
					if !sub {
						props
							.entry(ItemProp::Prerequisites)
							.or_insert(PropValue::Vec(Vec::new()))
							.insert(0, prereqs.clone());
					}
				}

				let desc = {
					let body = get_body(&mut str_pos, &span.extract, &captures);

					let mut lines: Vec<String> = Vec::new();
					for el in body.iter().rev() {
						if let Some(prop) = PROP_REGEX.captures(el) {
							if let (Some(prop_key), Some(prop_val)) = (prop.get(1), prop.get(2)) {
								let prop_key = ItemProp::from_str(prop_key.as_str())?;

								lines.push(prop_val.as_str().to_owned());
								lines.reverse();
								match prop_key {
									ItemProp::Prerequisites => {
										props.insert(
											ItemProp::Prerequisites,
											PropValue::Vec(
												to_paragraphs(lines)
													.join(" ")
													.split(", ")
													.map(str::to_owned)
													.collect(),
											),
										);
									}
									_ => {
										props
											.insert(prop_key, PropValue::Vec(to_paragraphs(lines)));
									}
								}
								lines = Vec::new();
							}
						} else {
							lines.push(el.clone());
						}
					}
					lines.reverse();
					lines
				};

				if let Some(sub_begin) = sub_begin {
					let mut desc = desc;
					desc.insert(0, normalize(sub_begin.as_str()));

					children.push(ParserSubItem {
						name: name.clone(),
						description: to_paragraphs(desc),
						properties: props,
					});
					continue;
				}

				desc
			}
			PageKind::MageSpell => todo!(),
			PageKind::Gift(_) => todo!(),
		};

		if let Some(match_) = captures.get(0) {
			let pos = match_.start();

			let page = span
				.page_ranges
				.iter()
				.find(|(_, range)| range.contains(&pos))
				.map_or(&0, |(p, _)| p);

			children.reverse();
			out.push(ParserItem {
				name: name.clone(),
				reference: BookReference(info.id, *page),
				description: to_paragraphs(desc),
				children,
				properties: props,
			});
			children = Vec::new();
		}
	}
	Ok(out)
}

fn normalize(str: &str) -> String {
	str.trim()
		.replace('\n', " ")
		.replace("  ", " ")
		.replace(['‘', '’'], "'")
}

fn filter_normalize(str: &str) -> Option<String> {
	let str = normalize(str);

	if str.is_empty() {
		None
	} else {
		Some(str)
	}
}
