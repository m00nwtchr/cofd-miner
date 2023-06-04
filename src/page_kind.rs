use std::{collections::HashMap, fs::File, io::Write};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PageKind {
	Merit,
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
	//
}

// pub enum ItemKind {
// 	// Top { children: Vec<Item> },
// }

fn is_empty<T>(vec: &Vec<T>) -> bool {
	vec.is_empty()
}

#[derive(Default, Serialize)]
pub struct Item {
	name: String,
	#[serde(default, skip_serializing_if = "is_empty")]
	children: Vec<Item>,
	desc: Vec<String>,
	// props: HashMap<String, String>,
}

impl PageKind {
	pub fn parse(&self, vec: &[String]) -> Vec<Item> {
		match self {
			PageKind::Merit => {
				let mut out = Vec::new();
				let str = vec.join("\n");

				// File::create("st2.txt").unwrap().write_all(str.as_bytes());

				let mut children = Vec::new();
				let mut str_pos = str.len();

				let matches: Vec<_> = MERIT_HEADER_REGEX.captures_iter(&str).collect();
				for captures in matches.into_iter().rev() {
					let name = captures.name("name").unwrap().as_str().trim();
					let cost = normalize(captures.name("cost").unwrap().as_str());
					let sub = captures.name("sub").is_some();

					let ltags = captures.name("ltags").map(|m| m.as_str()).unwrap_or("");
					let rtags = captures.name("rtags").map(|m| m.as_str()).unwrap_or("");

					let desc: Vec<String> = str[captures.get(0).unwrap().end()..str_pos]
						.split('\n')
						.filter_map(|str| filter_normalize(str))
						.collect();
					str_pos = captures.get(0).unwrap().start();

					// let desc = ;

					if !sub {
						out.push(Item {
							name: name.to_owned(),
							desc,
							children,
						});
						children = Vec::new();
					} else {
						children.push(Item {
							name: name.to_owned(),
							children: vec![],
							desc,
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
