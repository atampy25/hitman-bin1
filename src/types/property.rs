use std::fmt;

use bimap::BiMap;
use serde::{
	Deserialize, Serialize,
	de::{self, Visitor}
};

use crate::{
	de::{Bin1Deserialize, Bin1Deserializer, DeserializeError},
	ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError}
};

#[static_init::dynamic]
static PROPERTIES: BiMap<&'static str, u32> = include_str!("../../properties.txt")
	.lines()
	.zip(
		include_str!(concat!(env!("OUT_DIR"), "/properties-crc32.txt"))
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

impl Bin1Deserialize for PropertyID {
	const SIZE: usize = u32::SIZE;

	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		u32::read(de).map(Self)
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
