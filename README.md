# cofd-miner

A library for extraction of Chronicles of Darkness game material from pdfs you own. Very much work in progress.

## Example program: running from source

1. Install [Rust](https://www.rust-lang.org/learn/get-started)
2. Create `pdf` directory.
3. Place pdfs there.
4. Run `cargo run --bin pdf-extract --features pdf_extract`
5. Output json files in `out`

## Metadata editor

> `cargo meta-edit <pdf path>`

GUI application for creation of `meta/` files. (Only basic functionality for now)

## Supported books

See: [`meta/`](meta/) directory for list of supported books.
