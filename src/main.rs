use std::{env, fs};

use hitman_bin1::{
	de::deserialize,
	game::{
		h1::{STemplateEntity, STemplateEntityBlueprint as H1STemplateEntityBlueprint},
		h2::{
			STemplateEntityBlueprint as H2STemplateEntityBlueprint, STemplateEntityFactory as H2STemplateEntityFactory
		},
		h3::{
			STemplateEntityBlueprint as H3STemplateEntityBlueprint, STemplateEntityFactory as H3STemplateEntityFactory
		}
	},
	ser::serialize
};

fn main() {
	match env::args().nth(1).unwrap().as_ref() {
		"HM2016" => match env::args().nth(2).unwrap().as_ref() {
			"convert" => match env::args().nth(3).unwrap().as_ref() {
				"TEMP" => {
					let value: STemplateEntity = deserialize(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serde_json::to_vec(&value).unwrap()).unwrap();
				}

				"TBLU" => {
					let value: H1STemplateEntityBlueprint =
						deserialize(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serde_json::to_vec(&value).unwrap()).unwrap();
				}

				_ => panic!("3rd argument must be TEMP or TBLU")
			},

			"generate" => match env::args().nth(3).unwrap().as_ref() {
				"TEMP" => {
					let value: STemplateEntity =
						serde_json::from_slice(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serialize(&value).unwrap()).unwrap();
				}

				"TBLU" => {
					let value: H1STemplateEntityBlueprint =
						serde_json::from_slice(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serialize(&value).unwrap()).unwrap();
				}

				_ => panic!("3rd argument must be TEMP or TBLU")
			},

			_ => panic!("2nd argument must be convert or generate")
		},

		"HM2" => match env::args().nth(2).unwrap().as_ref() {
			"convert" => match env::args().nth(3).unwrap().as_ref() {
				"TEMP" => {
					let value: H2STemplateEntityFactory =
						deserialize(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serde_json::to_vec(&value).unwrap()).unwrap();
				}

				"TBLU" => {
					let value: H2STemplateEntityBlueprint =
						deserialize(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serde_json::to_vec(&value).unwrap()).unwrap();
				}

				_ => panic!("3rd argument must be TEMP or TBLU")
			},

			"generate" => match env::args().nth(3).unwrap().as_ref() {
				"TEMP" => {
					let value: H2STemplateEntityFactory =
						serde_json::from_slice(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serialize(&value).unwrap()).unwrap();
				}

				"TBLU" => {
					let value: H2STemplateEntityBlueprint =
						serde_json::from_slice(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serialize(&value).unwrap()).unwrap();
				}

				_ => panic!("3rd argument must be TEMP or TBLU")
			},

			_ => panic!("2nd argument must be convert or generate")
		},

		"HM3" => match env::args().nth(2).unwrap().as_ref() {
			"convert" => match env::args().nth(3).unwrap().as_ref() {
				"TEMP" => {
					let value: H3STemplateEntityFactory =
						deserialize(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serde_json::to_vec(&value).unwrap()).unwrap();
				}

				"TBLU" => {
					let value: H3STemplateEntityBlueprint =
						deserialize(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serde_json::to_vec(&value).unwrap()).unwrap();
				}

				_ => panic!("3rd argument must be TEMP or TBLU")
			},

			"generate" => match env::args().nth(3).unwrap().as_ref() {
				"TEMP" => {
					let value: H3STemplateEntityFactory =
						serde_json::from_slice(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serialize(&value).unwrap()).unwrap();
				}

				"TBLU" => {
					let value: H3STemplateEntityBlueprint =
						serde_json::from_slice(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serialize(&value).unwrap()).unwrap();
				}

				_ => panic!("3rd argument must be TEMP or TBLU")
			},

			_ => panic!("2nd argument must be convert or generate")
		},

		_ => panic!("1st argument must be HM1, HM2 or HM3")
	}
}
