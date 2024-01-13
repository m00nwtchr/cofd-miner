use cofd_miner::parse_book;
use cofd_schema::book::Book;

#[test]
fn roundtrip() -> anyhow::Result<()> {
	let book = parse_book("../pdf/Mage/Mage the Awakening 2e.pdf")?;

	let _book: Book = serde_json::de::from_str(&serde_json::ser::to_string(&book)?)?;
	println!("RON");
	// let book: Book = ron::de::from_str(&ron::ser::to_string(&book).unwrap()).unwrap();
	Ok(())
}