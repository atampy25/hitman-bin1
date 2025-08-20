use std::{fmt::Debug, sync::Arc};

use const_format::concatcp;
use ecow::EcoString;
use serde::Serialize;
use string_interner::{DefaultSymbol, StringInterner, backend::BucketBackend};

use crate::ser::{Aligned, Bin1Serialize};

pub trait StaticVariant {
	const TYPE_ID: &'static str;
}

#[dynex::dyn_trait]
pub trait Variant: VariantArc + Bin1Serialize + Send + Sync + Debug + Clone + PartialEq {
	fn type_id(&self, interner: &mut StringInterner<BucketBackend>) -> DefaultSymbol;

	/// Serialise this variant value into a serde_json Value. Does not include type information.
	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error>;

	/// Attempt to downcast this value as a Vec of Variants (which will all be of the same element type), allowing for generic operations on individual elements where the main Vec<T> type is unimportant.
	/// If this value is a Vec<T: Variant>, returns Some(Vec<&dyn Variant>), else returns None.
	fn as_vec(&self) -> Option<Vec<&dyn Variant>> {
		None
	}
}

pub trait VariantArc {
	fn into_inner_boxed_dyn(self: Arc<Self>) -> Option<Box<dyn Variant>>;
	fn unwrap_or_clone_boxed_dyn(self: Arc<Self>) -> Box<dyn Variant>;
	fn clone_underlying(&self) -> Arc<dyn Variant>;
}

impl<T: Variant + Clone> VariantArc for T {
	fn into_inner_boxed_dyn(self: Arc<Self>) -> Option<Box<dyn Variant>> {
		Arc::into_inner(self).map(|x| Box::new(x) as _)
	}

	fn unwrap_or_clone_boxed_dyn(self: Arc<Self>) -> Box<dyn Variant> {
		Box::new(Arc::unwrap_or_clone(self))
	}

	fn clone_underlying(&self) -> Arc<dyn Variant> {
		Arc::new(self.clone())
	}
}

impl dyn Variant {
	/// Get the type ID of this variant as a string. Inefficient; for repeated use it is better to reuse your own StringInterner with the [Variant::type_id] trait method directly.
	pub fn variant_type(&self) -> String {
		let mut interner = StringInterner::new();
		let type_id = Variant::type_id(self, &mut interner);
		interner.resolve(type_id).unwrap().to_owned()
	}

	pub fn is<T: Variant>(&self) -> bool {
		self.as_any().is::<T>()
	}

	pub fn into_boxed<T: Variant>(self: Box<dyn Variant>) -> Option<Box<T>> {
		self.as_any_box().downcast().ok()
	}

	pub fn into_unboxed<T: Variant>(self: Box<dyn Variant>) -> Option<T> {
		self.as_any_box().downcast().ok().map(|x| *x)
	}

	/// The first Option is the result of obtaining exclusive access to the Arc. The second Option is the result of downcasting into T.
	pub fn into_inner_boxed<T: Variant>(self: Arc<dyn Variant>) -> Option<Option<Box<T>>> {
		self.into_inner_boxed_dyn().map(|x| x.as_any_box().downcast().ok())
	}

	/// The first Option is the result of obtaining exclusive access to the Arc. The second Option is the result of downcasting into T.
	pub fn into_inner_unboxed<T: Variant>(self: Arc<dyn Variant>) -> Option<Option<T>> {
		self.into_inner_boxed_dyn()
			.map(|x| x.as_any_box().downcast().ok().map(|x| *x))
	}

	pub fn unwrap_or_clone_boxed<T: Variant>(self: Arc<dyn Variant>) -> Option<Box<T>> {
		self.unwrap_or_clone_boxed_dyn().as_any_box().downcast().ok()
	}

	pub fn unwrap_or_clone_unboxed<T: Variant>(self: Arc<dyn Variant>) -> Option<T> {
		self.unwrap_or_clone_boxed_dyn()
			.as_any_box()
			.downcast()
			.ok()
			.map(|x| *x)
	}

	pub fn as_ref<T: Variant>(&self) -> Option<&T> {
		self.as_any().downcast_ref()
	}

	pub fn as_mut<T: Variant>(&mut self) -> Option<&mut T> {
		self.as_any_mut().downcast_mut()
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

impl<
	T: Bin1Serialize + Aligned + Serialize + StaticVariant + Variant + Send + Sync + Clone + Debug + PartialEq + 'static
> Variant for Vec<T>
{
	fn type_id(&self, interner: &mut StringInterner<BucketBackend>) -> DefaultSymbol {
		interner.get_or_intern(format!("TArray<{}>", T::TYPE_ID))
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		serde_json::to_value(self)
	}

	fn as_vec(&self) -> Option<Vec<&dyn Variant>> {
		Some(self.iter().map(|x| x as &dyn Variant).collect())
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
