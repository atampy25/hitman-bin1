use std::fs;

use ecow::EcoString;
use hitman_bin1::{
	generated::h3::STemplateEntityFactory,
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
	let value: STemplateEntityFactory =
		serde_json::from_slice(&fs::read("00D52B924A8450DC.TEMP.json").unwrap()).unwrap();

	dbg!(&value);

	fs::write(
		"00D52B924A8450DC-roundtrip.TEMP.json",
		serde_json::to_vec(&value).unwrap()
	)
	.unwrap();

	fs::write("00D52B924A8450DC-roundtrip.TEMP", serialize(&value)?).unwrap();

	Ok(())
}
