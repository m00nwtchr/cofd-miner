use std::collections::HashMap;
use std::str::FromStr;

use convert_case::{Case, Casing};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

use cofd_schema::item::{Item, ItemProp, PropValue, SubItem};
use cofd_schema::prelude::DotRange;

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum PageKind {
	Merit(
		/**
			* Additional pre-requisites
			*/
		Option<String>,
	),
	MageSpell,
}

impl Default for PageKind {
	fn default() -> Self {
		Self::Merit(None)
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

	let v = vec.clone();

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
		.filter_map(|str| filter_normalize(str))
		.collect();
	*str_pos = captures.get(0).unwrap().start();

	body
}

impl PageKind {
	pub fn parse(&self, span: &str) -> Vec<Item> {
		let mut out = Vec::new();
		let mut str_pos = span.len();

		let mut children: Vec<SubItem> = Vec::new();

		let matches: Vec<_> = match self {
			PageKind::Merit(_) => MERIT_HEADER_REGEX.captures_iter(&span).collect(),
			PageKind::MageSpell => vec![],
		};
		for captures in matches.into_iter().rev() {
			let mut props = HashMap::new();

			let name = normalize(captures.name("name").unwrap().as_str());
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
						name.replace_range(i..(i + 1), "-");
					}
					name
				}
			} else {
				name.to_string()
			};

			let desc = match self {
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
						let strs: Vec<&str> = cost
							.as_str()
							.split(|c: char| c.is_whitespace() || c.eq(&','))
							.filter(|str| !str.is_empty() && !str.eq(&"or"))
							.collect();

						props.insert(
							ItemProp::DotRating,
							PropValue::DotRange(DotRange::from(strs.as_slice())),
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
						let body = get_body(&mut str_pos, span, &captures);

						let mut lines: Vec<String> = Vec::new();
						for el in body.iter().rev() {
							if let Some(prop) = PROP_REGEX.captures(el) {
								let prop_key =
									ItemProp::from_str(prop.get(1).unwrap().as_str()).unwrap();

								let prop_val = prop.get(2).unwrap().as_str().to_owned();

								lines.push(prop_val);
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
							} else {
								lines.push(el.clone());
							}
						}
						lines.reverse();
						lines
					};

					if sub {
						let mut desc = desc;
						desc.insert(0, normalize(sub_begin.unwrap().as_str()));

						children.push(SubItem {
							name: name.to_owned(),
							description: to_paragraphs(desc),
							properties: props,
						});
						continue;
					}

					desc
				}

				PageKind::MageSpell => {
					vec![]
				}
			};

			children.reverse();
			out.push(Item {
				name: name.to_owned(),
				desc: to_paragraphs(desc),
				children,
				props,
			});
			children = Vec::new();
		}
		out
	}
}

fn normalize(str: &str) -> String {
	str.trim()
		.replace('\n', " ")
		.replace("  ", " ")
		.replace('‘', "'")
		.replace('’', "'")
}

fn filter_normalize(str: &str) -> Option<String> {
	let str = normalize(str);

	if str.is_empty() {
		None
	} else {
		Some(str)
	}
}
