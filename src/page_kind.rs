use std::{collections::HashMap, fs::File, io::Write};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PageKind {
	Merit(Option<String>),
}

lazy_static! {
	static ref MERIT_HEADER_REGEX: Regex = Regex::new(r"(?xm)
		#[.\s]?
		(?P<name>[^\s.][^\n.]+)                 # Name
		\s?
		\(\s?
			(?: (?P<ltags> .+ ), )?             # Tags
			\s?
			(?P<cost>                           # Cost
				(?:          
					•{1,5}
					[,\s\+]*

					(?:to|or)?
					\s*
				)+
			)
			\s?
			(?: [,;] \s? (?P<rtags> [^\n]+ ) )? # Tags
		\s?\)
		\s?
		(?P<sub>:)?
	").unwrap();

	static ref PROP_REGEX: Regex = Regex::new(r"^(Prerequisite|Effect|Drawback)s?:$").unwrap();
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

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ItemProp {
	Prerequisites,
	Effects,
	Drawbacks,

	Cost,
	Tags,

	HeuristicPropExtraction,
}

impl ItemProp {
	fn by_name(str: &str) -> Option<Self> {
		match str.to_lowercase().as_str() {
			"effect" | "effects" => Some(Self::Effects),
			"prerequisite" | "prerequisites" => Some(Self::Prerequisites),
			"drawback" | "drawbacks" => Some(Self::Drawbacks),

			_ => None,
		}
	}
}

#[derive(Default, Serialize)]
pub struct Item {
	name: String,
	#[serde(default, skip_serializing_if = "is_empty")]
	children: Vec<Item>,
	desc: Vec<String>,
	#[serde(default, skip_serializing_if = "is_empty_map")]
	props: HashMap<ItemProp, Vec<String>>,
}

impl PageKind {
	pub fn parse(&self, vec: &[String]) -> Vec<Item> {
		match self {
			PageKind::Merit(additional_prereqs) => {
				let mut out = Vec::new();
				let str = vec.join("\n");

				// File::create("st2.txt").unwrap().write_all(str.as_bytes());

				let mut children = Vec::new();
				let mut str_pos = str.len();

				let matches: Vec<_> = MERIT_HEADER_REGEX.captures_iter(&str).collect();
				for captures in matches.into_iter().rev() {
					let name = captures.name("name").unwrap().as_str().trim();
					let cost = captures.name("cost");
					let sub = captures.name("sub").is_some();

					let ltags = captures.name("ltags").map(|m| m.as_str());
					let rtags = captures.name("rtags").map(|m| m.as_str());

					let body: Vec<String> = str[captures.get(0).unwrap().end()..str_pos]
						.split('\n')
						.filter_map(|str| filter_normalize(str))
						.collect();
					str_pos = captures.get(0).unwrap().start();

					let mut props = HashMap::new();
					let mut v: Vec<String> = Vec::new();

					for el in body.iter().rev() {
						if let Some(prop) = PROP_REGEX.captures(el) {
							let prop = ItemProp::by_name(prop.get(1).unwrap().as_str()).unwrap();

							if prop != ItemProp::Prerequisites || !props.is_empty() {
								v.reverse();
								props.insert(prop, v);
								v = Vec::new();
							} else if prop == ItemProp::Prerequisites && props.is_empty() {
								v.reverse();
								let mut a = Vec::new();
								let mut b = Vec::new();

								let mut flag = true;

								for item in v {
									/*if item.contains(" ") && !item.contains(",") {
										flag = false;
									} else */
									if item.split_whitespace().count() > 3 {
										flag = false;
									} else if item.chars().all(|f| f.eq(&'•')) {
									}

									if !flag && item.contains("Combat: ") {
										a.push(item);
									} else {
										if flag {
											a.push(item);
										} else {
											b.push(item);
										}
									}
								}

								props.insert(prop, a);
								props.insert(
									ItemProp::HeuristicPropExtraction,
									vec!["true".to_owned()],
								);

								v = b;
								v.reverse()
							}
						} else {
							v.push(el.clone());
						}
					}
					v.reverse();
					let desc = v;

					if let Some(t) = ltags.or(rtags) {
						let v: Vec<String> = t.split(", ").map(String::from).collect();
						props.insert(ItemProp::Tags, v);
					}

					if let Some(cost) = cost {
						props.insert(ItemProp::Cost, vec![normalize(cost.as_str())]);
					}

					if let Some(prereqs) = additional_prereqs {
						props
							.entry(ItemProp::Prerequisites)
							.or_default()
							.insert(0, prereqs.clone());
					}

					if !sub {
						out.push(Item {
							name: name.to_owned(),
							desc,
							children,
							props,
						});
						children = Vec::new();
					} else {
						children.push(Item {
							name: name.to_owned(),
							children: vec![],
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
