use cofd_miner::parse_book;
use std::path::PathBuf;

fn main() {
	let args: Vec<_> = std::env::args().collect();
	let path = PathBuf::from(args.get(1).unwrap());

	let b = parse_book(path).unwrap();
	print!("{}", serde_json::ser::to_string_pretty(&b).unwrap());
}
