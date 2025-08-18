use std::{
	collections::HashMap,
	fmt::{self, Debug},
	ops::{Deref, DerefMut}
};

use const_format::concatcp;
use ecow::EcoString;
use serde::{Deserialize, Serialize, de::DeserializeOwned, ser::SerializeStruct};
use string_interner::{DefaultSymbol, StringInterner, backend::BucketBackend};

use crate::ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError};

pub trait StaticVariant {
	const TYPE_ID: &'static str;
}

#[dynex::dyn_trait]
pub trait Variant: Bin1Serialize + Send + Sync + Debug + Clone + PartialEq {
	fn type_id(&self, interner: &mut StringInterner<BucketBackend>) -> DefaultSymbol;
	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error>;
}

pub trait DeserializeVariant: Send + Sync {
	fn type_id(&self) -> &str;
	fn deserialize(&self, type_id: &str, value: serde_json::Value) -> Result<Box<dyn Variant>, String>;
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

impl<T: StaticVariant + Variant + DeserializeOwned + 'static + Send + Sync> DeserializeVariant
	for VariantDeserializer<T>
{
	fn type_id(&self) -> &str {
		T::TYPE_ID
	}

	fn deserialize(&self, type_id: &str, value: serde_json::Value) -> Result<Box<dyn Variant>, String> {
		if type_id != T::TYPE_ID {
			return Err(format!("Cannot deserialize {} into {}", type_id, T::TYPE_ID));
		}

		serde_json::from_value::<T>(value)
			.map(|v| Box::new(v) as Box<dyn Variant>)
			.map_err(|e| format!("{e}"))
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
				.deserialize(&type_id, value)
				.map_err(serde::de::Error::custom)?
				.into())
		} else {
			Err(serde::de::Error::custom(format!("unknown type ID: {}", type_id)))
		}
	}
}

macro_rules! impl_primitive {
	($ty:ty, $type_id:literal) => {
		impl StaticVariant for $ty {
			const TYPE_ID: &'static str = $type_id;
		}

		impl Variant for $ty {
			fn type_id(&self, interner: &mut StringInterner<BucketBackend>) -> DefaultSymbol {
				interner.get_or_intern_static(Self::TYPE_ID)
			}

			fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
				Ok((*self).into())
			}
		}

		impl StaticVariant for Vec<$ty> {
			const TYPE_ID: &'static str = concatcp!("TArray<", $type_id, ">");
		}

		impl StaticVariant for Vec<Vec<$ty>> {
			const TYPE_ID: &'static str = concatcp!("TArray<TArray<", $type_id, ">>");
		}

		inventory::submit!(&VariantDeserializer::<$ty>::new() as &dyn DeserializeVariant);
		inventory::submit!(&VariantDeserializer::<Vec<$ty>>::new() as &dyn DeserializeVariant);
		inventory::submit!(&VariantDeserializer::<Vec<Vec<$ty>>>::new() as &dyn DeserializeVariant);
	};
}

impl_primitive!(u8, "uint8");
impl_primitive!(u16, "uint16");
impl_primitive!(u32, "uint32");
impl_primitive!(u64, "uint64");

impl_primitive!(i8, "int8");
impl_primitive!(i16, "int16");
impl_primitive!(i32, "int32");
impl_primitive!(i64, "int64");

impl_primitive!(f32, "float32");
impl_primitive!(f64, "float64");

impl_primitive!(bool, "bool");

impl StaticVariant for () {
	const TYPE_ID: &'static str = "void";
}

impl StaticVariant for Vec<()> {
	const TYPE_ID: &'static str = "TArray<void>";
}

impl StaticVariant for Vec<Vec<()>> {
	const TYPE_ID: &'static str = "TArray<TArray<void>>";
}

impl Variant for () {
	fn type_id(&self, interner: &mut StringInterner<BucketBackend>) -> DefaultSymbol {
		interner.get_or_intern_static(Self::TYPE_ID)
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		Ok(serde_json::Value::Null)
	}
}

inventory::submit!(&VariantDeserializer::<()>::new() as &dyn DeserializeVariant);
inventory::submit!(&VariantDeserializer::<Vec<()>>::new() as &dyn DeserializeVariant);
inventory::submit!(&VariantDeserializer::<Vec<Vec<()>>>::new() as &dyn DeserializeVariant);

impl<T: Bin1Serialize + Aligned + Serialize + StaticVariant + Send + Sync + Clone + Debug + PartialEq + 'static> Variant
	for Vec<T>
{
	fn type_id(&self, interner: &mut StringInterner<BucketBackend>) -> DefaultSymbol {
		interner.get_or_intern(format!("TArray<{}>", T::TYPE_ID))
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		serde_json::to_value(self)
	}
}

impl StaticVariant for EcoString {
	const TYPE_ID: &'static str = "ZString";
}

impl StaticVariant for Vec<EcoString> {
	const TYPE_ID: &'static str = "TArray<ZString>";
}

impl StaticVariant for Vec<Vec<EcoString>> {
	const TYPE_ID: &'static str = "TArray<TArray<ZString>>";
}

impl Variant for EcoString {
	fn type_id(&self, interner: &mut StringInterner<BucketBackend>) -> DefaultSymbol {
		interner.get_or_intern_static(Self::TYPE_ID)
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		Ok(serde_json::Value::String(self.as_str().into()))
	}
}

inventory::submit!(&VariantDeserializer::<EcoString>::new() as &dyn DeserializeVariant);
inventory::submit!(&VariantDeserializer::<Vec<EcoString>>::new() as &dyn DeserializeVariant);
inventory::submit!(&VariantDeserializer::<Vec<Vec<EcoString>>>::new() as &dyn DeserializeVariant);
