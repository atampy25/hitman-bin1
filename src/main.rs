use std::{env, fs};

use hitman_bin1::{
	generated::h3::{STemplateEntityFactory, STemplateEntityBlueprint},
	ser::{SerializeError, serialize}
};

fn main() -> Result<(), SerializeError> {
	match env::args().nth(2).unwrap().as_ref() {
		"TEMP" => {
			let value: STemplateEntityFactory =
				serde_json::from_slice(&fs::read(format!("{}.TEMP.json", env::args().nth(1).unwrap())).unwrap()).unwrap();

			fs::write(format!("{}-roundtrip.TEMP", env::args().nth(1).unwrap()), serialize(&value)?).unwrap();
		}

		"TBLU" => {
			let value: STemplateEntityBlueprint =
				serde_json::from_slice(&fs::read(format!("{}.TBLU.json", env::args().nth(1).unwrap())).unwrap()).unwrap();

			fs::write(format!("{}-roundtrip.TBLU", env::args().nth(1).unwrap()), serialize(&value)?).unwrap();
		}

		_ => panic!("Invalid type")
	}

	Ok(())
}
