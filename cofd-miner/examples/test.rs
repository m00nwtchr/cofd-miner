use std::path::PathBuf;

use cofd_pdf_extract::parse_book;

fn main() {
	let args: Vec<_> = std::env::args().collect();
	let path = PathBuf::from(args.get(1).unwrap());

	println!("{:?}", parse_book(path).unwrap());
}
