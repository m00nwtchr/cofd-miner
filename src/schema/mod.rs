use std::ops::Range;

mod prerequisites;

enum DotRange {
	Set(Vec<u8>),
	Range(Range<u8>),
}

impl From<&str> for DotRange {
	fn from(value: &str) -> Self {
		let mut op = None;

		value.split_whitespace().map(|f| {
			
		});

		DotRange::Set(vec![])
	}
}

impl DotRange {}
