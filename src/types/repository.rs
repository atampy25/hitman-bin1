use serde::{Deserialize, Serialize};
use string_interner::{DefaultSymbol, StringInterner, backend::BucketBackend};
use thiserror::Error;

use crate::{
	de::Bin1Deserialize,
	ser::Bin1Serialize,
	types::variant::{StaticVariant, Variant}
};

use crate as hitman_bin1;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Bin1Serialize, Bin1Deserialize)]
#[serde(try_from = "&str", into = "String")]
pub struct ZRepositoryID {
	pub data_1: u32,
	pub data_2: u16,
	pub data_3: u16,
	pub data_4: [u8; 8]
}

#[derive(Error, Debug)]
pub enum RepositoryIdError {
	#[error("failed to parse repository ID component as hex")]
	ParseError(#[from] std::num::ParseIntError),

	#[error("not enough dash separated parts")]
	NotEnoughParts,

	#[error("not enough characters in part")]
	NotEnoughChars
}

impl TryFrom<&str> for ZRepositoryID {
	type Error = RepositoryIdError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let mut parts = value.split('-');

		Ok(Self {
			data_1: u32::from_str_radix(parts.next().ok_or(RepositoryIdError::NotEnoughParts)?, 16)?,
			data_2: u16::from_str_radix(parts.next().ok_or(RepositoryIdError::NotEnoughParts)?, 16)?,
			data_3: u16::from_str_radix(parts.next().ok_or(RepositoryIdError::NotEnoughParts)?, 16)?,
			data_4: {
				let mut data_4 = [0u8; 8];

				let mut part = parts.next().ok_or(RepositoryIdError::NotEnoughParts)?.chars();
				for item in data_4.iter_mut().take(2) {
					let char1 = part.next().ok_or(RepositoryIdError::NotEnoughChars)?;
					let char2 = part.next().ok_or(RepositoryIdError::NotEnoughChars)?;
					*item = u8::from_str_radix(&[char1, char2].into_iter().collect::<String>(), 16)?;
				}

				let mut part = parts.next().ok_or(RepositoryIdError::NotEnoughParts)?.chars();
				for item in data_4.iter_mut().skip(2) {
					let char1 = part.next().ok_or(RepositoryIdError::NotEnoughChars)?;
					let char2 = part.next().ok_or(RepositoryIdError::NotEnoughChars)?;
					*item = u8::from_str_radix(&[char1, char2].into_iter().collect::<String>(), 16)?;
				}

				data_4
			}
		})
	}
}

impl From<ZRepositoryID> for String {
	fn from(value: ZRepositoryID) -> Self {
		format!(
			"{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
			value.data_1,
			value.data_2,
			value.data_3,
			value.data_4[0],
			value.data_4[1],
			value.data_4[2],
			value.data_4[3],
			value.data_4[4],
			value.data_4[5],
			value.data_4[6],
			value.data_4[7],
		)
	}
}

impl StaticVariant for ZRepositoryID {
	const TYPE_ID: &'static str = "ZRepositoryID";
}

impl StaticVariant for Vec<ZRepositoryID> {
	const TYPE_ID: &'static str = "TArray<ZRepositoryID>";
}

impl StaticVariant for Vec<Vec<ZRepositoryID>> {
	const TYPE_ID: &'static str = "TArray<TArray<ZRepositoryID>>";
}

impl Variant for ZRepositoryID {
	fn type_id(&self, interner: &mut StringInterner<BucketBackend>) -> DefaultSymbol {
		interner.get_or_intern_static(Self::TYPE_ID)
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		serde_json::to_value(self)
	}
}
