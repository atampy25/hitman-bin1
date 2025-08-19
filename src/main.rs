use std::{env, fs};

use hitman_bin1::{
	de::deserialize,
	game::h3::{STemplateEntityBlueprint, STemplateEntityFactory},
	ser::serialize
};

fn main() {
	match env::args().nth(2).unwrap().as_ref() {
		"convert" => match env::args().nth(3).unwrap().as_ref() {
			"TEMP" => {
				let value: STemplateEntityFactory =
					deserialize(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

				fs::write(env::args().nth(5).unwrap(), serde_json::to_vec(&value).unwrap()).unwrap();
			}

			"TBLU" => {
				let value: STemplateEntityBlueprint =
					deserialize(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

				fs::write(env::args().nth(5).unwrap(), serde_json::to_vec(&value).unwrap()).unwrap();
			}

			_ => panic!("3rd argument must be TEMP or TBLU")
		},

		"generate" => match env::args().nth(3).unwrap().as_ref() {
			"TEMP" => {
				let value: STemplateEntityFactory =
					serde_json::from_slice(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

				fs::write(env::args().nth(5).unwrap(), serialize(&value).unwrap()).unwrap();
			}

			"TBLU" => {
				let value: STemplateEntityBlueprint =
					serde_json::from_slice(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

				fs::write(env::args().nth(5).unwrap(), serialize(&value).unwrap()).unwrap();
			}

			_ => panic!("3rd argument must be TEMP or TBLU")
		},

		_ => panic!("2nd argument must be convert or generate")
	}
}
