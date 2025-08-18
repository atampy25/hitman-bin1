use serde::{Deserialize, Serialize};
use string_interner::{DefaultSymbol, StringInterner, backend::BucketBackend};

use crate::{
	ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError},
	types::variant::{DeserializeVariant, StaticVariant, Variant, VariantDeserializer}
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZRuntimeResourceID {
	#[serde(rename = "m_IDHigh")]
	pub high: u32,

	#[serde(rename = "m_IDLow")]
	pub low: u32
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
		ser.write_runtime_resource_id(self.high, self.low);

		Ok(())
	}
}

inventory::submit!(&VariantDeserializer::<ZRuntimeResourceID>::new() as &dyn DeserializeVariant);
inventory::submit!(&VariantDeserializer::<Vec<ZRuntimeResourceID>>::new() as &dyn DeserializeVariant);
inventory::submit!(&VariantDeserializer::<Vec<Vec<ZRuntimeResourceID>>>::new() as &dyn DeserializeVariant);
