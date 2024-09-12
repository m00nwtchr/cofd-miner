use std::str::FromStr;

use cofd_schema::{
	book::BookReference,
	dot_range::dots_to_num,
	item::{
		gift::{Facet, Gift, GiftKind, Moon, Other},
		Item,
	},
	splat::werewolf::Renown,
};

use crate::MultiMap;

#[allow(clippy::type_complexity)]
pub fn parse_gifts(map: MultiMap) -> anyhow::Result<(Vec<Gift<Moon>>, Vec<Gift<Other>>)> {
	let mut moon_gifts = Vec::new();
	let mut gifts = Vec::new();

	for (cat, vec) in map {
		let mut moon_gift = None;
		let mut gift = None;
		let kind = if cat.contains("Moon") {
			GiftKind::Moon
		} else if cat.contains("Shadow") {
			GiftKind::Shadow
		} else if cat.contains("Wolf") {
			GiftKind::Wolf
		} else {
			unreachable!()
		};

		for vec in vec {
			if vec.len() == 1 {
				if let Some(g) = gift.take() {
					gifts.push(g);
				}
				if let Some(g) = moon_gift.take() {
					moon_gifts.push(g);
				}

				if let Some(name) = vec.first() {
					if !name.contains('(') {
						match kind {
							GiftKind::Moon => {
								moon_gift = Some(Gift {
									name: name.to_owned(),
									facets: Vec::new(),
									kind: GiftKind::Moon,
								});
							}
							GiftKind::Shadow | GiftKind::Wolf => {
								gift = Some(Gift {
									name: name.to_owned(),
									facets: Vec::new(),
									kind,
								});
							}
						}
					}
				}
			} else {
				let name = vec[0].clone();
				let str = vec[1].clone();

				// let cost = vec[2].clone();
				// let pool = vec[3].clone();
				// let action = vec[4].clone();
				// let duration = vec[5].clone();

				let description = vec[6].clone();
				let reference = &vec[7];

				match kind {
					GiftKind::Moon => {
						if let Some(moon_gift) = &mut moon_gift {
							moon_gift.facets.push(Item {
								name,
								reference: BookReference::from_str(reference)?,
								description: vec![description],
								inner: Facet {
									inner: Moon {
										level: dots_to_num(&str).unwrap_or(0),
										auspice: cofd_schema::splat::werewolf::Auspice::Cahalith,
									},
									action: /*if !(cost.is_empty()
										&& pool.is_empty() && action.is_empty()
										&& duration.is_empty())
									{
										Some(ActionFields {
											cost: vec!,
											dice_pool: todo!(),
											action: todo!(),
											duration: todo!(),
										})
									} else { */
										None
									//},
								},
								effects: Vec::new(),
							});
						}
					}
					GiftKind::Shadow | GiftKind::Wolf => {
						if let Some(gift) = &mut gift {
							gift.facets.push(Item {
								name,
								reference: BookReference::from_str(reference)?,
								description: vec![description],
								effects: Vec::new(),
								inner: Facet {
									inner: Other {
										renown: Renown::from_str(&str)?,
									},
									action: None,
								},
							});
						}
					}
				}
			}
		}

		match kind {
			GiftKind::Moon => {
				if let Some(g) = moon_gift {
					moon_gifts.push(g);
				}
			}
			GiftKind::Shadow | GiftKind::Wolf => {
				if let Some(g) = gift {
					gifts.push(g);
				}
			}
		}
	}

	Ok((moon_gifts, gifts))
}

// fn gift_name_to_id(name: &str) -> &str {
// 	if name.contains("of") {
// 		name.split(' ').last().unwrap()
// 	} else {
// 		let next = name.split(' ').next().unwrap();
// 		if next.contains('\'') {
// 			next.strip_suffix("\'s").unwrap()
// 		} else {
// 			next
// 		}
// 	}
// }

// fn facet_name_to_id(name: &str) -> String {
// 	name.replace(['\'', ','], "")
// 		.to_case(convert_case::Case::Pascal)
// }
