use std::fs;

use ecow::EcoString;
use hitman_bin1::{
	ser::{SerializeError, serialize},
	types::{pointers::Owned, variant::ZVariant}
};
use hitman_bin1_derive::Bin1Serialize;

#[derive(Bin1Serialize)]
pub struct XYZ {
	value: Vec<Owned<EcoString>>,
	value2: ZVariant
}

fn main() -> Result<(), SerializeError> {
	let mut val = XYZ {
		value: vec![],
		value2: Box::new(vec![2u8, 4, 6])
	};

	val.value.push(Owned::new("Hello world!".into()));
	let abc: EcoString = "Long string!!!!!!!".into();
	val.value.push(Owned::new(abc.clone()));
	val.value.push(Owned::new(abc));
	val.value.push(Owned::new("Something else".into()));

	fs::write("ser.bin", serialize(&val)?)?;

	Ok(())
}
