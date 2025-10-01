use std::mem::MaybeUninit;

use tryvial::try_fn;

use crate::{
	de::Bin1Deserialize,
	ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError}
};

impl<T: Aligned> Aligned for [T] {
	const ALIGNMENT: usize = T::ALIGNMENT;
}

impl<T: Aligned, const N: usize> Aligned for [T; N] {
	const ALIGNMENT: usize = T::ALIGNMENT;
}

/// Serialisation of arrays as TFixedArray values (elements directly written aligned, with no length specified).
impl<T: Bin1Serialize + Aligned> Bin1Serialize for [T] {
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

impl<T: Bin1Serialize + Aligned, const N: usize> Bin1Serialize for [T; N] {
	fn alignment(&self) -> usize {
		(self as &[T]).alignment()
	}

	fn write(&self, ser: &mut crate::ser::Bin1Serializer) -> Result<(), crate::ser::SerializeError> {
		(self as &[T]).write(ser)
	}

	fn resolve(&self, ser: &mut crate::ser::Bin1Serializer) -> Result<(), crate::ser::SerializeError> {
		(self as &[T]).resolve(ser)
	}
}

impl<T: Bin1Deserialize, const N: usize> Bin1Deserialize for [T; N] {
	const SIZE: usize = T::SIZE * N;

	fn read(de: &mut crate::de::Bin1Deserializer) -> Result<Self, crate::de::DeserializeError> {
		let mut result = [const { MaybeUninit::uninit() }; N];

		for elem in &mut result {
			de.align_to(T::ALIGNMENT)?;
			elem.write(T::read(de)?);
			de.align_to(T::ALIGNMENT)?;
		}

		Ok(unsafe { std::mem::transmute_copy(&result) })
	}
}

impl<T> Aligned for Vec<T> {
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
			ser.write_pointee(start_id, Some(end_id), self.as_slice())?;
		}

		Ok(())
	}
}

impl<T: Bin1Deserialize> Bin1Deserialize for Vec<T> {
	const SIZE: usize = 8 * 3;

	#[try_fn]
	fn read(de: &mut crate::de::Bin1Deserializer) -> Result<Self, crate::de::DeserializeError> {
		de.align_to(8)?;
		let start = de.read_u64()?;
		let end = de.read_u64()?;

		if start == u64::MAX || end == u64::MAX {
			de.seek_relative(8)?;
			return Ok(Vec::new());
		}

		let len = (end as usize - start as usize) / T::SIZE;
		let pos = de.position();

		de.seek_from_start(start + 0x10)?;
		let mut result = Vec::with_capacity(len);
		for _ in 0..len {
			de.align_to(T::ALIGNMENT)?;
			result.push(T::read(de)?);
		}

		// Seek past the allocation end pointer
		de.seek_from_start(pos + 8)?;

		result
	}
}

#[allow(non_snake_case)]
pub mod TArrayRef {
	use crate::{
		de::Bin1Deserialize,
		ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError}
	};

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
				ser.write_pointee(start_id, Some(end_id), self.0)?;
			}

			Ok(())
		}
	}

	pub struct De<T: Bin1Deserialize>(Vec<T>);

	impl<T: Bin1Deserialize> From<De<T>> for Vec<T> {
		fn from(value: De<T>) -> Self {
			value.0
		}
	}

	impl<T: Bin1Deserialize> Aligned for De<T> {
		const ALIGNMENT: usize = 8;
	}

	impl<T: Bin1Deserialize> Bin1Deserialize for De<T> {
		const SIZE: usize = 8 * 2;

		#[tryvial::try_fn]
		fn read(de: &mut crate::de::Bin1Deserializer) -> Result<Self, crate::de::DeserializeError> {
			de.align_to(8)?;
			let start = de.read_u64()?;
			let end = de.read_u64()?;

			if start == u64::MAX || end == u64::MAX {
				return Ok(De(Vec::new()));
			}

			let len = (end as usize - start as usize) / T::SIZE;
			let pos = de.position();

			de.seek_from_start(start + 0x10)?;

			let mut result = Vec::with_capacity(len);
			for _ in 0..len {
				de.align_to(T::ALIGNMENT)?;
				result.push(T::read(de)?);
			}

			de.seek_from_start(pos)?;

			De(result)
		}
	}
}
