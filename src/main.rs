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
	match env::args().nth(1).as_deref() {
		Some("HM2016") => match env::args().nth(2).as_deref() {
			Some("convert") => match env::args().nth(3).as_deref() {
				Some("TEMP") => {
					let value: STemplateEntity = deserialize(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serde_json::to_vec(&value).unwrap()).unwrap();
				}

				Some("TBLU") => {
					let value: H1STemplateEntityBlueprint =
						deserialize(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serde_json::to_vec(&value).unwrap()).unwrap();
				}

				_ => panic!("3rd argument must be TEMP or TBLU")
			},

			Some("generate") => match env::args().nth(3).as_deref() {
				Some("TEMP") => {
					let value: STemplateEntity =
						serde_json::from_slice(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serialize(&value).unwrap()).unwrap();
				}

				Some("TBLU") => {
					let value: H1STemplateEntityBlueprint =
						serde_json::from_slice(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serialize(&value).unwrap()).unwrap();
				}

				_ => panic!("3rd argument must be TEMP or TBLU")
			},

			_ => panic!("2nd argument must be convert or generate")
		},

		Some("HM2") => match env::args().nth(2).as_deref() {
			Some("convert") => match env::args().nth(3).as_deref() {
				Some("TEMP") => {
					let value: H2STemplateEntityFactory =
						deserialize(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serde_json::to_vec(&value).unwrap()).unwrap();
				}

				Some("TBLU") => {
					let value: H2STemplateEntityBlueprint =
						deserialize(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serde_json::to_vec(&value).unwrap()).unwrap();
				}

				_ => panic!("3rd argument must be TEMP or TBLU")
			},

			Some("generate") => match env::args().nth(3).as_deref() {
				Some("TEMP") => {
					let value: H2STemplateEntityFactory =
						serde_json::from_slice(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serialize(&value).unwrap()).unwrap();
				}

				Some("TBLU") => {
					let value: H2STemplateEntityBlueprint =
						serde_json::from_slice(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serialize(&value).unwrap()).unwrap();
				}

				_ => panic!("3rd argument must be TEMP or TBLU")
			},

			_ => panic!("2nd argument must be convert or generate")
		},

		Some("HM3") => match env::args().nth(2).as_deref() {
			Some("convert") => match env::args().nth(3).as_deref() {
				Some("TEMP") => {
					let value: H3STemplateEntityFactory =
						deserialize(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serde_json::to_vec(&value).unwrap()).unwrap();
				}

				Some("TBLU") => {
					let value: H3STemplateEntityBlueprint =
						deserialize(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serde_json::to_vec(&value).unwrap()).unwrap();
				}

				_ => panic!("3rd argument must be TEMP or TBLU")
			},

			Some("generate") => match env::args().nth(3).as_deref() {
				Some("TEMP") => {
					let value: H3STemplateEntityFactory =
						serde_json::from_slice(&fs::read(env::args().nth(4).unwrap()).unwrap()).unwrap();

					fs::write(env::args().nth(5).unwrap(), serialize(&value).unwrap()).unwrap();
				}

				Some("TBLU") => {
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
