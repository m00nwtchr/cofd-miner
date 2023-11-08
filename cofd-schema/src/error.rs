use std::num::ParseIntError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
	#[error(transparent)]
	StrumError(#[from] strum::ParseError),
	#[error(transparent)]
	IntError(#[from] ParseIntError),
	#[error("The provided data was in a wrong format: {0}")]
	BadFormat(String),
}

#[derive(Error, Debug)]
pub enum SchemaError {
	#[error("Parsing error")]
	ParseError(ParseError),
}
