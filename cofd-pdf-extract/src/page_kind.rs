use std::collections::HashMap;

use cofd_schema::prelude::DotRange;
use convert_case::{Case, Casing};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PageKind {
	Merit(Option<String>),
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

// pub enum ItemKind {
// 	// Top { children: Vec<Item> },
// }

fn is_empty<T>(vec: &Vec<T>) -> bool {
	vec.is_empty()
}

fn is_empty_map<K, V>(map: &HashMap<K, V>) -> bool {
	map.is_empty()
}

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ItemProp {
	Description,
	DotRating,
	Tags,

	Prerequisites,
	StyleTags,
	Cost,
	DicePool,
	Action,
	Duration,
	Effects,
	Drawbacks,
	Notes,
}

impl ItemProp {
	fn by_name(str: &str) -> Option<Self> {
		match str.to_lowercase().as_str() {
			"prerequisite" | "prerequisites" => Some(Self::Prerequisites),
			"style tag" | "style tags" => Some(Self::StyleTags),
			"cost" => Some(Self::Cost),
			"dice pool" => Some(Self::DicePool),
			"action" => Some(Self::Action),
			"duration" => Some(Self::Duration),
			"effect" | "effects" => Some(Self::Effects),
			"drawback" | "drawbacks" => Some(Self::Drawbacks),
			"note" | "notes" => Some(Self::Notes),
			_ => None,
		}
	}
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropValue {
	Vec(Vec<String>),
	Bool(bool),
	DotRange(DotRange),
}

impl PropValue {
	pub fn insert(&mut self, index: usize, element: String) {
		if let PropValue::Vec(vec) = self {
			vec.insert(index, element)
		}
	}
}

#[derive(Default, Serialize)]
pub struct SubItem {
	name: String,
	desc: Vec<String>,
	#[serde(default, skip_serializing_if = "is_empty_map")]
	props: HashMap<ItemProp, PropValue>,
}

#[derive(Default, Serialize)]
pub struct Item {
	name: String,
	#[serde(default, skip_serializing_if = "is_empty")]
	children: Vec<SubItem>,
	desc: Vec<String>,
	#[serde(default, skip_serializing_if = "is_empty_map")]
	props: HashMap<ItemProp, PropValue>,
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

		paragraph.push_str(if !line.ends_with("God-") {
			line.trim_end_matches('-')
		} else {
			flag = true;
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

impl PageKind {
	pub fn parse(&self, span: &str) -> Vec<Item> {
		match self {
			PageKind::Merit(additional_prereqs) => {
				let mut out = Vec::new();

				let mut children = Vec::new();
				let mut str_pos = span.len();

				let matches: Vec<_> = MERIT_HEADER_REGEX.captures_iter(&span).collect();
				for captures in matches.into_iter().rev() {
					let name = captures.name("name").unwrap().as_str().trim();
					let name =
						if name.chars().filter(|f| f.is_uppercase()).count() >= (name.len() / 2) {
							name.split_whitespace()
								.map(|f| {
									let f = f.to_lowercase();
									if !f.eq("to") && !f.eq("or") && !f.eq("the") && !f.eq("of") {
										f.to_case(Case::Title)
									} else {
										f.to_string()
									}
								})
								.fold(String::new(), |a, b| a + &b + " ")
								.trim()
								.to_string()
						} else {
							name.to_string()
						};

					let cost = captures.name("cost");

					let sub = captures.name("sub").is_some();
					let sub_begin = captures.name("subbegin");

					let ltags = captures.name("ltags").map(|m| m.as_str());
					let rtags = captures.name("rtags").map(|m| m.as_str());

					let mut props = HashMap::new();
					let desc = {
						let body: Vec<String> = span[captures.get(0).unwrap().end()..str_pos]
							.split('\n')
							.filter_map(|str| filter_normalize(str))
							.collect();
						str_pos = captures.get(0).unwrap().start();

						let mut v: Vec<String> = Vec::new();
						for el in body.iter().rev() {
							if let Some(prop) = PROP_REGEX.captures(el) {
								let prop_key =
									ItemProp::by_name(prop.get(1).unwrap().as_str()).unwrap();

								let prop_val = prop.get(2).unwrap().as_str().to_owned();

								if prop_key == ItemProp::Prerequisites {
									props.insert(
										ItemProp::Prerequisites,
										PropValue::Vec(
											prop_val.split(", ").map(str::to_owned).collect(),
										),
									);

									if !v.is_empty() {
										v.reverse();
										props.insert(
											ItemProp::Description,
											PropValue::Vec(to_paragraphs(v)),
										);
										v = Vec::new();
									}
								} else {
									v.push(prop_val);
									v.reverse();
									props.insert(prop_key, PropValue::Vec(to_paragraphs(v)));
									v = Vec::new();
								}
							} else {
								v.push(el.clone());
							}
						}
						v.reverse();
						// let desc = v;
						to_paragraphs(v)
					};

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
						props
							.entry(ItemProp::Prerequisites)
							.or_insert(PropValue::Vec(Vec::new()))
							.insert(0, prereqs.clone());
					}

					if !sub {
						children.reverse();
						out.push(Item {
							name: name.to_owned(),
							desc,
							children,
							props,
						});
						children = Vec::new();
					} else {
						let mut desc = desc;
						desc.insert(0, normalize(sub_begin.unwrap().as_str()));
						children.push(SubItem {
							name: name.to_owned(),
							desc,
							props,
						});
					}
				}

				out
			}
		}
	}
}

fn normalize(str: &str) -> String {
	str.trim().replace('\n', " ").replace("  ", " ")
}

fn filter_normalize(str: &str) -> Option<String> {
	let str = normalize(str);

	if str.is_empty() {
		None
	} else {
		Some(str)
	}
}
