use std::{fs::File, io::Write};

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
		(?P<name>[^\s.][^\n.]+) # Name
		\s
		\(\s?
			#(?P<tags>.+,)*      # Tags
			#\s*
			(                    # Cost
				(?:          
					•{1,5}
					[,\s\+]*

					(?:to)?
					\s*
				)+
			)
			#\s?
			#.*?
		\n?\)
		:?
	").unwrap();
	// static ref MERIT_HEADER_REGEX: Regex =
	// Regex::new(r"(?P<name>[A-Za-z ]+) \(").unwrap();
}

impl PageKind {
	pub fn parse(&self, vec: &[String]) -> Vec<String> {
		match self {
			PageKind::Merit => {
				let mut out = Vec::new();
				// for str in vec {
				let str = vec.join("\n");
				// File::create("txt.txt")
				// 	.unwrap()
				// 	.write_all(str.as_bytes())
				// 	.unwrap();

				// if let Some(captures) = {
				for captures in MERIT_HEADER_REGEX.captures_iter(&str) {
					let mut iter = captures.iter();
					iter.next();
					for capture in iter {
						if let Some(capture) = capture {
							println!("{} ", capture.as_str());
						}
					}
					println!("");
					out.push(captures.name("name").unwrap().as_str().to_owned());
				}

				// out.push(str.clone());
				// }
				// }
				out
			}
		}
	}
}
