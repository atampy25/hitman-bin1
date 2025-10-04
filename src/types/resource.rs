use serde::{Deserialize, Serialize};
use string_interner::{DefaultSymbol, StringInterner, backend::BucketBackend};
use tryvial::try_fn;

use crate::{
	de::{Bin1Deserialize, Bin1Deserializer, DeserializeError},
	ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError},
	types::variant::{StaticVariant, Variant}
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZRuntimeResourceID {
	#[serde(rename = "m_IDHigh")]
	pub id_high: u32,

	#[serde(rename = "m_IDLow")]
	pub id_low: u32
}

impl StaticVariant for ZRuntimeResourceID {
	const TYPE_ID: &'static str = "ZRuntimeResourceID";
}

impl StaticVariant for Vec<ZRuntimeResourceID> {
	const TYPE_ID: &'static str = "TArray<ZRuntimeResourceID>";
}

impl StaticVariant for Vec<Vec<ZRuntimeResourceID>> {
	const TYPE_ID: &'static str = "TArray<TArray<ZRuntimeResourceID>>";
}

impl Variant for ZRuntimeResourceID {
	fn type_id(&self, interner: &mut StringInterner<BucketBackend>) -> DefaultSymbol {
		interner.get_or_intern_static(Self::TYPE_ID)
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		serde_json::to_value(self)
	}
}

impl Aligned for ZRuntimeResourceID {
	const ALIGNMENT: usize = 8;
}

impl Bin1Serialize for ZRuntimeResourceID {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		ser.write_runtime_resource_id(self.id_high, self.id_low);

		Ok(())
	}
}

impl Bin1Deserialize for ZRuntimeResourceID {
	const SIZE: usize = 8;

	#[try_fn]
	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		let id_high = u32::read(de)?;
		let id_low = u32::read(de)?;
		Self { id_high, id_low }
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TResourcePtr {
	#[serde(rename = "m_IDHigh")]
	pub id_high: u32,

	#[serde(rename = "m_IDLow")]
	pub id_low: u32
}

impl StaticVariant for TResourcePtr {
	const TYPE_ID: &'static str = "TResourcePtr";
}

impl StaticVariant for Vec<TResourcePtr> {
	const TYPE_ID: &'static str = "TArray<TResourcePtr>";
}

impl StaticVariant for Vec<Vec<TResourcePtr>> {
	const TYPE_ID: &'static str = "TArray<TArray<TResourcePtr>>";
}

impl Variant for TResourcePtr {
	fn type_id(&self, interner: &mut StringInterner<BucketBackend>) -> DefaultSymbol {
		interner.get_or_intern_static(Self::TYPE_ID)
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		serde_json::to_value(self)
	}
}

impl Aligned for TResourcePtr {
	const ALIGNMENT: usize = 8;
}

impl Bin1Serialize for TResourcePtr {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		ser.write_resource_ptr(self.id_high, self.id_low);

		Ok(())
	}
}

impl Bin1Deserialize for TResourcePtr {
	const SIZE: usize = 8;

	#[try_fn]
	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		let id_high = u32::read(de)?;
		let id_low = u32::read(de)?;
		Self { id_high, id_low }
	}
}
