use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum MeritTag {
	Style,
	#[strum(to_string = "Supernatural Merit")]
	Supernatural,
}
