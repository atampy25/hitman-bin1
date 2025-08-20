#![allow(non_camel_case_types)]

use std::{
	collections::HashMap,
	fmt::{self, Debug},
	ops::{Deref, DerefMut}
};

use serde::{Deserialize, Serialize, de::DeserializeOwned, ser::SerializeStruct};
use string_interner::{DefaultSymbol, StringInterner, backend::BucketBackend};
use tryvial::try_fn;

use crate::types::{property::PropertyID, resource::ZRuntimeResourceID};

pub trait DeserializeVariant: Send + Sync {
	fn type_id(&self) -> &str;
	fn deserialize_serde(&self, type_id: &str, value: serde_json::Value) -> Result<Box<dyn Variant>, String>;
	fn deserialize_bin1(&self, type_id: &str, de: &mut Bin1Deserializer) -> Result<Box<dyn Variant>, DeserializeError>;
}

pub struct VariantDeserializer<T: StaticVariant + Variant + DeserializeOwned + 'static + Send + Sync>(
	std::marker::PhantomData<T>
);

impl<T: StaticVariant + Variant + DeserializeOwned + 'static + Send + Sync> VariantDeserializer<T> {
	#[allow(clippy::new_without_default)]
	pub const fn new() -> Self {
		Self(std::marker::PhantomData)
	}
}

impl<T: StaticVariant + Variant + Bin1Deserialize + DeserializeOwned + 'static + Send + Sync> DeserializeVariant
	for VariantDeserializer<T>
{
	fn type_id(&self) -> &str {
		T::TYPE_ID
	}

	fn deserialize_serde(&self, type_id: &str, value: serde_json::Value) -> Result<Box<dyn Variant>, String> {
		if type_id != T::TYPE_ID {
			return Err(format!("Cannot deserialize {} into {}", type_id, T::TYPE_ID));
		}

		serde_json::from_value::<T>(value)
			.map(|v| Box::new(v) as Box<dyn Variant>)
			.map_err(|e| format!("{e}"))
	}

	fn deserialize_bin1(&self, type_id: &str, de: &mut Bin1Deserializer) -> Result<Box<dyn Variant>, DeserializeError> {
		if type_id != T::TYPE_ID {
			return Err(DeserializeError::TypeMismatch {
				expected: T::TYPE_ID,
				found: type_id.to_owned()
			});
		}

		de.read::<T>().map(|v| Box::new(v) as Box<dyn Variant>)
	}
}

inventory::collect!(&'static dyn DeserializeVariant);

#[static_init::dynamic]
static DESERIALIZERS: HashMap<&'static str, &'static dyn DeserializeVariant> =
	inventory::iter::<&'static dyn DeserializeVariant>
		.into_iter()
		.map(|&x| (x.type_id(), x))
		.collect();

#[derive(Clone, dynex::PartialEqFix)]
pub struct ZVariant {
	value: Box<dyn Variant>
}

impl Deref for ZVariant {
	type Target = dyn Variant;

	fn deref(&self) -> &Self::Target {
		&*self.value
	}
}

impl DerefMut for ZVariant {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut *self.value
	}
}

impl From<Box<dyn Variant>> for ZVariant {
	fn from(value: Box<dyn Variant>) -> Self {
		Self { value }
	}
}

impl ZVariant {
	pub fn new<T: Variant>(value: T) -> Self {
		Self { value: Box::new(value) }
	}

	pub fn into_inner(self) -> Box<dyn Variant> {
		self.value
	}

	pub fn into_boxed<T: Variant>(self) -> Option<Box<T>> {
		self.value.as_any_box().downcast().ok()
	}

	pub fn into_unboxed<T: Variant>(self) -> Option<T> {
		self.value.as_any_box().downcast().ok().map(|x| *x)
	}
}

impl Debug for ZVariant {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_tuple("ZVariant").field(&self.value).finish()
	}
}

impl Aligned for ZVariant {
	const ALIGNMENT: usize = 8;
}

impl Bin1Serialize for ZVariant {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		let type_id = Variant::type_id(&*self.value, ser.interner());
		ser.write_type(type_id);

		if Variant::type_id(&*self.value, ser.interner()) == ser.interner().get_or_intern_static("void") {
			ser.write_pointer(u64::MAX); // void type has no data
		} else {
			let pointer_id = (&*self.value) as *const _ as *const () as u64 | 0xBEEF000000000000;
			ser.write_pointer(pointer_id);
		}

		Ok(())
	}

	fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		if Variant::type_id(&*self.value, ser.interner()) != ser.interner().get_or_intern_static("void") {
			let pointer_id = (&*self.value) as *const _ as *const () as u64 | 0xBEEF000000000000;
			ser.write_pointee(pointer_id, None, &*self.value)?;
		}

		Ok(())
	}
}

impl Bin1Deserialize for ZVariant {
	const SIZE: usize = 8 * 2;

	#[try_fn]
	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		let type_id = de.read_type()?;

		if type_id == "void" {
			de.seek_relative(8)?; // skip pointer
			Self::new(())
		} else {
			de.align_to(8)?;
			let ptr = de.read_u64()?;
			let pos = de.position();

			de.seek_from_start(ptr + 0x10)?;

			let result = DESERIALIZERS
				.get(type_id.as_str())
				.ok_or_else(|| DeserializeError::UnknownType(type_id.to_string()))?
				.deserialize_bin1(&type_id, de)?;

			de.seek_from_start(pos)?;

			result.into()
		}
	}
}

impl Serialize for ZVariant {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
		S::Error: serde::ser::Error
	{
		let mut ser = serializer.serialize_struct("ZVariant", 2)?;
		let mut interner = StringInterner::new();
		let type_id = Variant::type_id(&*self.value, &mut interner);
		ser.serialize_field("$type", interner.resolve(type_id).unwrap())?;
		ser.serialize_field(
			"$val",
			&self.value.to_serde().map_err(<S::Error as serde::ser::Error>::custom)?
		)?;
		ser.end()
	}
}

impl<'de> Deserialize<'de> for ZVariant {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>
	{
		let mut type_id = None;
		let mut value = None;

		let map = serde_json::Map::<String, serde_json::Value>::deserialize(deserializer)?;
		for (key, val) in map {
			match key.as_str() {
				"$type" => {
					type_id = Some(
						val.as_str()
							.ok_or_else(|| serde::de::Error::custom("$type must be string"))?
							.to_owned()
					);
				}

				"$val" => {
					value = Some(val);
				}

				_ => return Err(serde::de::Error::unknown_field(&key, &["$type", "$val"]))
			}
		}

		let type_id = type_id.ok_or_else(|| serde::de::Error::missing_field("$type"))?;
		let value = value.ok_or_else(|| serde::de::Error::missing_field("$val"))?;

		if let Some(deserializer) = DESERIALIZERS.get(type_id.as_str()) {
			Ok(deserializer
				.deserialize_serde(&type_id, value)
				.map_err(serde::de::Error::custom)?
				.into())
		} else {
			Err(serde::de::Error::custom(format!("unknown type ID: {}", type_id)))
		}
	}
}

impl StaticVariant for ZVariant {
	const TYPE_ID: &'static str = "ZVariant";
}

impl Variant for ZVariant {
	fn type_id(&self, interner: &mut StringInterner<BucketBackend>) -> DefaultSymbol {
		interner.get_or_intern_static(Self::TYPE_ID)
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		serde_json::to_value(self)
	}
}

impl StaticVariant for Vec<ZVariant> {
	const TYPE_ID: &'static str = "TArray<ZVariant>";
}

impl StaticVariant for Vec<Vec<ZVariant>> {
	const TYPE_ID: &'static str = "TArray<TArray<ZVariant>>";
}

macro_rules! submit {
	($ty:ty) => {
		inventory::submit!(&VariantDeserializer::<$ty>::new() as &dyn DeserializeVariant);
		inventory::submit!(&VariantDeserializer::<Vec<$ty>>::new() as &dyn DeserializeVariant);
		inventory::submit!(&VariantDeserializer::<Vec<Vec<$ty>>>::new() as &dyn DeserializeVariant);
	};
}

submit!(ZVariant);

submit!(u8);
submit!(u16);
submit!(u32);
submit!(u64);
submit!(i8);
submit!(i16);
submit!(i32);
submit!(i64);
submit!(f32);
submit!(f64);
submit!(bool);
submit!(());
submit!(EcoString);
submit!(SEntityTemplateProperty);
submit!(ZRuntimeResourceID);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Bin1Serialize, Bin1Deserialize)]
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

submit!(SEntityTemplateProperty);

impl StaticVariant for (EcoString, ZVariant) {
	const TYPE_ID: &'static str = "TPair<ZString,ZVariant>";
}

impl StaticVariant for Vec<(EcoString, ZVariant)> {
	const TYPE_ID: &'static str = "TArray<TPair<ZString,ZVariant>>";
}

impl StaticVariant for Vec<Vec<(EcoString, ZVariant)>> {
	const TYPE_ID: &'static str = "TArray<TArray<TPair<ZString,ZVariant>>>";
}

submit!((EcoString, ZVariant));

include!(concat!(env!("OUT_DIR"), "/h3.rs"));
