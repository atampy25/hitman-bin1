use std::sync::{Arc, Weak};

use crate::ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError};

macro_rules! impl_primitive {
	($ty:ty, $alignment:literal, $type_id:literal) => {
		impl Aligned for $ty {
			const ALIGNMENT: usize = $alignment;
		}

		impl Bin1Serialize for $ty {
			fn alignment(&self) -> usize {
				$alignment
			}

			fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
				ser.write_unaligned(&self.to_le_bytes());
				Ok(())
			}
		}
	};

	($ty:ty, $alignment:literal) => {
		impl Aligned for $ty {
			const ALIGNMENT: usize = $alignment;
		}

		impl Bin1Serialize for $ty {
			fn alignment(&self) -> usize {
				$alignment
			}

			fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
				ser.write_unaligned(&self.to_le_bytes());
				Ok(())
			}
		}
	};
}

impl_primitive!(u8, 1, "uint8");
impl_primitive!(u16, 2, "uint16");
impl_primitive!(u32, 4, "uint32");
impl_primitive!(u64, 8, "uint64");
impl_primitive!(usize, 8);

impl_primitive!(i8, 1, "int8");
impl_primitive!(i16, 2, "int16");
impl_primitive!(i32, 4, "int32");
impl_primitive!(i64, 8, "int64");
impl_primitive!(isize, 8);

impl_primitive!(f32, 4, "float32");
impl_primitive!(f64, 8, "float64");

impl Aligned for bool {
	const ALIGNMENT: usize = 1;
}

impl Bin1Serialize for bool {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		ser.write_unaligned(&[*self as u8]);
		Ok(())
	}
}

impl<T: Bin1Serialize> Aligned for Arc<T> {
	const ALIGNMENT: usize = 8;
}

impl<T: Bin1Serialize> Bin1Serialize for Arc<T> {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		let pointer_id = Arc::as_ptr(self) as u64;
		ser.write_pointer(pointer_id);
		Ok(())
	}

	fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		let pointer_id = Arc::as_ptr(self) as u64;
		ser.write_pointee(pointer_id, None, self.as_ref())
	}
}

impl<T: Bin1Serialize> Aligned for Option<Arc<T>> {
	const ALIGNMENT: usize = 8;
}

impl<T: Bin1Serialize> Bin1Serialize for Option<Arc<T>> {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		if let Some(value) = self {
			value.write_aligned(ser)?;
		} else {
			ser.write_pointer(u64::MAX);
		}

		Ok(())
	}

	fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		if let Some(value) = self {
			value.resolve(ser)?;
		} else {
			ser.write_pointer(u64::MAX);
		}

		Ok(())
	}
}

impl<T: Bin1Serialize> Aligned for Weak<T> {
	const ALIGNMENT: usize = 8;
}

impl<T: Bin1Serialize> Bin1Serialize for Weak<T> {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		let value = self.upgrade();
		value.write_aligned(ser)
	}

	fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		let value = self.upgrade();
		value.resolve(ser)
	}
}

impl<T: Aligned, U> Aligned for (T, U) {
	const ALIGNMENT: usize = T::ALIGNMENT;
}

impl<T: Bin1Serialize, U: Bin1Serialize> Bin1Serialize for (T, U) {
	fn alignment(&self) -> usize {
		self.0.alignment()
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		self.0.write(ser)?;
		self.1.write_aligned(ser)?;
		Ok(())
	}

	fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		self.0.resolve(ser)?;
		self.1.resolve(ser)?;
		Ok(())
	}
}

impl Aligned for () {
	const ALIGNMENT: usize = 1;
}

impl Bin1Serialize for () {
	fn alignment(&self) -> usize {
		1
	}

	fn write(&self, _ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		Ok(())
	}
}
