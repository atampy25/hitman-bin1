use std::ops::{Deref, DerefMut};

use rand::Rng;

use crate::ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError};

pub struct Owned<T: Bin1Serialize> {
	pub value: T,
	identity: u64
}

impl<T: Bin1Serialize> Owned<T> {
	pub fn new(value: T) -> Self {
		Self {
			value,
			identity: rand::rng().random()
		}
	}
}

impl<T: Bin1Serialize> From<T> for Owned<T> {
	fn from(value: T) -> Self {
		Self {
			value,
			identity: rand::rng().random()
		}
	}
}

impl<T: Bin1Serialize> Deref for Owned<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<T: Bin1Serialize> DerefMut for Owned<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}

impl<T: Bin1Serialize> Aligned for Owned<T> {
	const ALIGNMENT: usize = 8;
}

impl<T: Bin1Serialize> Bin1Serialize for Owned<T> {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		ser.write_pointer(self.identity);
		Ok(())
	}

	fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		ser.write_pointee(self.identity, &self.value)
	}
}
