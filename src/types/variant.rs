use string_interner::{DefaultSymbol, StringInterner, backend::BucketBackend};

use crate::ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError};

pub trait StaticVariant {
	const TYPE_ID: &'static str;
}

pub trait Variant: Bin1Serialize {
	fn type_id(&self, interner: &mut StringInterner<BucketBackend>) -> DefaultSymbol;
	fn to_serde(&self) -> serde_json::Value;
}

pub type ZVariant = Box<dyn Variant>;

impl Aligned for ZVariant {
	const ALIGNMENT: usize = 8;
}

impl Bin1Serialize for ZVariant {
	fn alignment(&self) -> usize {
		(**self).alignment()
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		let pointer_id = (&**self) as *const dyn Variant as *const () as u64;
		let type_id = self.type_id(ser.interner());
		ser.write_type(type_id);
		ser.write_pointer(pointer_id);
		Ok(())
	}

	fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		let pointer_id = (&**self) as *const dyn Variant as *const () as u64;
		ser.write_pointee(pointer_id, &**self)
	}
}
