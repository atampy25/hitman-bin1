use std::fmt::Debug;

use const_format::concatcp;
use ecow::EcoString;
use serde::Serialize;
use string_interner::{DefaultSymbol, StringInterner, backend::BucketBackend};

use crate::ser::{Aligned, Bin1Serialize};

pub trait StaticVariant {
	const TYPE_ID: &'static str;
}

#[dynex::dyn_trait]
pub trait Variant: Bin1Serialize + Send + Sync + Debug + Clone + PartialEq {
	fn type_id(&self, interner: &mut StringInterner<BucketBackend>) -> DefaultSymbol;
	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error>;
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

impl<
	T: Bin1Serialize + Aligned + Serialize + StaticVariant + Send + Sync + Clone + Debug + PartialEq + 'static,
	U: Bin1Serialize + Aligned + Serialize + StaticVariant + Send + Sync + Clone + Debug + PartialEq + 'static
> Variant for (T, U)
{
	fn type_id(&self, interner: &mut StringInterner<BucketBackend>) -> DefaultSymbol {
		interner.get_or_intern(format!("TPair<{},{}>", T::TYPE_ID, U::TYPE_ID))
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		serde_json::to_value(self)
	}
}

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
