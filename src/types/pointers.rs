use std::ops::{Deref, DerefMut};

use rand::Rng;

use crate::{
	de::Bin1Deserialize,
	ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError}
};

pub struct Owned<T> {
	pub value: T,
	identity: u64
}

impl<T> Owned<T> {
	pub fn new(value: T) -> Self {
		Self {
			value,
			identity: rand::rng().random()
		}
	}
}

impl<T> From<T> for Owned<T> {
	fn from(value: T) -> Self {
		Self {
			value,
			identity: rand::rng().random()
		}
	}
}

impl<T> Deref for Owned<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<T> DerefMut for Owned<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}

impl<T> Aligned for Owned<T> {
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
		ser.write_pointee(self.identity, None, &self.value)
	}
}

impl<T: Bin1Deserialize> Bin1Deserialize for Owned<T> {
	const SIZE: usize = 8;

	#[tryvial::try_fn]
	fn read(de: &mut crate::de::Bin1Deserializer) -> Result<Self, crate::de::DeserializeError> {
		let ptr = de.read_u64()?;
		let pos = de.position();

		de.seek_from_start(ptr + 0x10)?;
		de.align_to(T::ALIGNMENT)?;
		let value = T::read(de)?;
		de.seek_from_start(pos)?;

		Self {
			value,
			identity: rand::rng().random()
		}
	}
}
