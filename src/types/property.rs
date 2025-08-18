use std::fmt;

use bimap::BiMap;
use serde::{
	Deserialize, Serialize,
	de::{self, Visitor}
};
use string_interner::{DefaultSymbol, StringInterner, backend::BucketBackend};

use crate::{
	ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError},
	types::variant::{DeserializeVariant, StaticVariant, Variant, VariantDeserializer, ZVariant}
};

use crate as hitman_bin1;

#[static_init::dynamic]
static PROPERTIES: BiMap<&'static str, u32> = include_str!("../../properties.txt")
	.lines()
	.zip(
		include_str!("../../properties-crc32.txt")
			.lines()
			.map(|x| x.parse().unwrap())
	)
	.collect();

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PropertyID(pub u32);

impl PropertyID {
	pub fn from_known(name: &str) -> Option<Self> {
		PROPERTIES.get_by_left(name).map(|&x| x.into())
	}

	pub fn as_name(&self) -> Option<&'static str> {
		PROPERTIES.get_by_right(&self.0).copied()
	}
}

impl Aligned for PropertyID {
	const ALIGNMENT: usize = u32::ALIGNMENT;
}

impl Bin1Serialize for PropertyID {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		self.0.write(ser)
	}
}

impl From<u32> for PropertyID {
	fn from(value: u32) -> Self {
		Self(value)
	}
}

impl From<PropertyID> for u32 {
	fn from(value: PropertyID) -> Self {
		value.0
	}
}

impl Serialize for PropertyID {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer
	{
		if let Some(name) = self.as_name() {
			serializer.serialize_str(name)
		} else {
			serializer.serialize_u32(self.0)
		}
	}
}

impl<'de> Deserialize<'de> for PropertyID {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>
	{
		struct PropertyIDVisitor;

		impl<'de> Visitor<'de> for PropertyIDVisitor {
			type Value = PropertyID;

			fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
				write!(f, "a property id as a string or integer")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: de::Error
			{
				PropertyID::from_known(v).ok_or_else(|| E::custom("unknown property name"))
			}

			fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
			where
				E: de::Error
			{
				Ok(v.into())
			}

			fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
			where
				E: de::Error
			{
				if v <= u32::MAX as u64 {
					Ok((v as u32).into())
				} else {
					Err(E::custom("integer out of range for PropertyID"))
				}
			}

			fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
			where
				E: de::Error
			{
				if v >= 0 && v <= u32::MAX as i64 {
					Ok((v as u32).into())
				} else {
					Err(E::custom("integer out of range for PropertyID"))
				}
			}
		}

		deserializer.deserialize_any(PropertyIDVisitor)
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Bin1Serialize)]
pub struct SEntityTemplateProperty {
	#[serde(rename = "nPropertyID")]
	pub property_id: PropertyID,

	#[serde(rename = "value")]
	#[bin1(pad = 4)]
	pub value: ZVariant
}

impl StaticVariant for SEntityTemplateProperty {
	const TYPE_ID: &'static str = "SEntityTemplateProperty";
}

impl StaticVariant for Vec<SEntityTemplateProperty> {
	const TYPE_ID: &'static str = "TArray<SEntityTemplateProperty>";
}

impl StaticVariant for Vec<Vec<SEntityTemplateProperty>> {
	const TYPE_ID: &'static str = "TArray<TArray<SEntityTemplateProperty>>";
}

impl Variant for SEntityTemplateProperty {
	fn type_id(&self, interner: &mut StringInterner<BucketBackend>) -> DefaultSymbol {
		interner.get_or_intern_static(Self::TYPE_ID)
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		serde_json::to_value(self)
	}
}

inventory::submit!(&VariantDeserializer::<SEntityTemplateProperty>::new() as &dyn DeserializeVariant);
inventory::submit!(&VariantDeserializer::<Vec<SEntityTemplateProperty>>::new() as &dyn DeserializeVariant);
inventory::submit!(&VariantDeserializer::<Vec<Vec<SEntityTemplateProperty>>>::new() as &dyn DeserializeVariant);
