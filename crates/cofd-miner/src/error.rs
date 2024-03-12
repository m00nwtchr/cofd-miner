use thiserror::Error;

#[derive(Error, Debug)]
pub enum CofDMinerError {
	#[error("No such metadata definition found")]
	NoSuchMeta,
}
