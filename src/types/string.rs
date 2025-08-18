use ecow::EcoString;

use crate::ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError};

impl Aligned for EcoString {
	const ALIGNMENT: usize = 8;
}

impl Bin1Serialize for EcoString {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		let length = (self.len() as i32) | 0x40000000;
		let pointer_id = self.as_ptr() as u64;

		length.write_aligned(ser)?;
		ser.write_pointer(pointer_id);

		Ok(())
	}

	fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		let pointer_id = self.as_ptr() as u64;
		ser.write_pointee(pointer_id, None, &self.as_bytes())?;
		ser.write_unaligned(&[0]); // Null terminator

		Ok(())
	}
}

#[allow(non_snake_case)]
pub mod CString {
	use crate::ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError};
	pub struct Ser<'a>(pub &'a str);

	impl<'a> From<&'a str> for Ser<'a> {
		fn from(value: &'a str) -> Self {
			Self(value)
		}
	}

	impl Aligned for Ser<'_> {
		const ALIGNMENT: usize = 1;
	}

	impl Bin1Serialize for Ser<'_> {
		fn alignment(&self) -> usize {
			Self::ALIGNMENT
		}

		fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
			ser.write_unaligned(self.0.as_bytes());
			ser.write_unaligned(&[0]); // Null terminator
			Ok(())
		}
	}
}
