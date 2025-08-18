use crate::ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError};

impl<T: Bin1Serialize + Aligned> Aligned for &[T] {
	const ALIGNMENT: usize = T::ALIGNMENT;
}

/// Direct serialisation of slices as arrays with no length value.
impl<T: Bin1Serialize + Aligned> Bin1Serialize for &[T] {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut crate::ser::Bin1Serializer) -> Result<(), crate::ser::SerializeError> {
		for item in *self {
			item.write_aligned(ser)?;
		}

		Ok(())
	}

	fn resolve(&self, ser: &mut crate::ser::Bin1Serializer) -> Result<(), crate::ser::SerializeError> {
		for item in *self {
			item.resolve(ser)?;
		}

		Ok(())
	}
}

impl<T: Bin1Serialize + Aligned, const N: usize> Aligned for [T; N] {
	const ALIGNMENT: usize = T::ALIGNMENT;
}

/// Serialisation of arrays as TFixedArray values (no length specified).
impl<T: Bin1Serialize + Aligned, const N: usize> Bin1Serialize for [T; N] {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut crate::ser::Bin1Serializer) -> Result<(), crate::ser::SerializeError> {
		for item in self {
			item.write_aligned(ser)?;
		}

		Ok(())
	}

	fn resolve(&self, ser: &mut crate::ser::Bin1Serializer) -> Result<(), crate::ser::SerializeError> {
		for item in self {
			item.resolve(ser)?;
		}

		Ok(())
	}
}

impl<T: Bin1Serialize> Aligned for Vec<T> {
	const ALIGNMENT: usize = 8;
}

/// Serialisation of Vec<T> in TArray format, with pointers and length.
impl<T: Bin1Serialize + Aligned> Bin1Serialize for Vec<T> {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		if self.is_empty() {
			ser.write_pointer(u64::MAX);
			ser.write_pointer(u64::MAX);
			ser.write_pointer(u64::MAX);
		} else {
			let start_id = self.as_ptr() as u64 | 0xABCD000000000000; // fake pointers to avoid colliding with actual data
			let end_id = start_id | 0xCAFE000000000000;
			ser.write_pointer(start_id);
			ser.write_pointer(end_id);
			ser.write_pointer(end_id); // allocation end, which in serialisation/deserialisation is the same as the end
		}

		Ok(())
	}

	fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		if !self.is_empty() {
			let start_id = self.as_ptr() as u64 | 0xABCD000000000000;
			let end_id = start_id | 0xCAFE000000000000;
			ser.write_pointee(start_id, Some(end_id), &self.as_slice())?;
		}

		Ok(())
	}
}

#[allow(non_snake_case)]
pub mod TArrayRef {
	use crate::ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError};

	pub struct Ser<'a, T: Bin1Serialize>(pub &'a [T]);

	impl<'a, T: Bin1Serialize> From<&'a [T]> for Ser<'a, T> {
		fn from(value: &'a [T]) -> Self {
			Self(value)
		}
	}

	impl<'a, T: Bin1Serialize> Aligned for Ser<'a, T> {
		const ALIGNMENT: usize = 8;
	}

	impl<'a, T: Bin1Serialize + Aligned> Bin1Serialize for Ser<'a, T> {
		fn alignment(&self) -> usize {
			Self::ALIGNMENT
		}

		fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
			if self.0.is_empty() {
				ser.write_pointer(u64::MAX);
				ser.write_pointer(u64::MAX);
			} else {
				let start_id = self.0.as_ptr() as u64;
				let end_id = self.0.as_ptr_range().end as u64;
				ser.write_pointer(start_id);
				ser.write_pointer(end_id);
			}

			Ok(())
		}

		fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
			if !self.0.is_empty() {
				let start_id = self.0.as_ptr() as u64;
				let end_id = self.0.as_ptr_range().end as u64;
				ser.write_pointee(start_id, Some(end_id), &self.0)?;
			}

			Ok(())
		}
	}
}
