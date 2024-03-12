use std::str::FromStr;

use crate::MultiMap;
use cofd_schema::{
	book::BookReference,
	item::{
		merit::{Merit, MeritTag},
		Item,
	},
	prelude::DotRange,
	prerequisites::Prerequisites,
};

pub fn parse_merits(map: MultiMap) -> anyhow::Result<Vec<Item<Merit>>> {
	let mut merits = Vec::new();

	for (cat, vec) in map {
		for mut vec in vec {
			let name = vec[0].to_owned();
			let dot_rating = DotRange::from_str(&vec[1]).unwrap();
			let prerequisites = Prerequisites::from_str(&vec[2]).unwrap();
			let description = vec.remove(3);
			let book = &vec[3];

			let mut tags = Vec::new();

			match cat.as_str() {
				"Mental Merits" => tags.push(MeritTag::Mental),
				"Social Merits" => tags.push(MeritTag::Social),
				"Physical Merits" => tags.push(MeritTag::Physical),
				"Fighting Merits" => tags.push(MeritTag::Fighting),
				"Supernatural Merits" => tags.push(MeritTag::Supernatural),
				_ => {}
			}

			merits.push(Item {
				name,
				reference: BookReference::from_str(book).unwrap_or_else(|er| {
					println!("{book}, {er}");
					panic!()
				}),
				description: vec![description],
				effects: Vec::new(),
				inner: Merit {
					dot_rating,
					prerequisites,
					tags,
					drawbacks: Vec::new(),
					children: Vec::new(),
					action: None,
					notes: Vec::new(),
				},
			});
		}
	}

	Ok(merits)
}
