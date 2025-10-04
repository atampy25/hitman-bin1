use std::{env, fs};

use ecow::EcoString;
use hitman_bin1::{de::deserialize, ser::serialize};

#[cfg(all(feature = "h1"))]
use hitman_bin1::game::h1;

#[cfg(all(feature = "h2"))]
use hitman_bin1::game::h2;

#[cfg(all(feature = "h3"))]
use hitman_bin1::game::h3;

macro_rules! impl_convert {
	($resource_type:ident, $feature:literal, $ty:literal, $res:ty) => {
		#[cfg(feature = $feature)]
		if $resource_type == $ty {
			let value: $res =
				deserialize(&fs::read(env::args().nth(4).expect("4th argument must be input path")).unwrap()).unwrap();

			fs::write(
				env::args().nth(5).expect("4th argument must be output path"),
				serde_json::to_vec(&value).unwrap()
			)
			.unwrap();
		}
	};

	($resource_type:ident, $ty:literal, $res:ty) => {
		#[cfg(feature = $ty)]
		if $resource_type == $ty {
			let value: $res =
				deserialize(&fs::read(env::args().nth(4).expect("4th argument must be input path")).unwrap()).unwrap();

			fs::write(
				env::args().nth(5).expect("4th argument must be output path"),
				serde_json::to_vec(&value).unwrap()
			)
			.unwrap();
		}
	};
}

macro_rules! impl_generate {
	($resource_type:ident, $feature:literal, $ty:literal, $res:ty) => {
		#[cfg(feature = $feature)]
		if $resource_type == $ty {
			let value: $res = serde_json::from_slice(
				&fs::read(env::args().nth(4).expect("4th argument must be input path")).unwrap()
			)
			.unwrap();

			fs::write(
				env::args().nth(5).expect("4th argument must be output path"),
				serialize(&value).unwrap()
			)
			.unwrap();
		}
	};

	($resource_type:ident, $ty:literal, $res:ty) => {
		#[cfg(feature = $ty)]
		if $resource_type == $ty {
			let value: $res = serde_json::from_slice(
				&fs::read(env::args().nth(4).expect("4th argument must be input path")).unwrap()
			)
			.unwrap();

			fs::write(
				env::args().nth(5).expect("4th argument must be output path"),
				serialize(&value).unwrap()
			)
			.unwrap();
		}
	};
}

macro_rules! impl_all {
	($resource_type:ident, h1, $impl:ident) => {
		impl_all!(generic, $resource_type, h1, $impl);

		$impl!($resource_type, "TEMP", h1::STemplateEntity);
	};

	($resource_type:ident, h2, $impl:ident) => {
		impl_all!(generic, $resource_type, h2, $impl);

		$impl!($resource_type, "TEMP", h2::STemplateEntityFactory);
		$impl!($resource_type, "ECPB", h2::SExtendedCppEntityBlueprint);
	};

	($resource_type:ident, h3, $impl:ident) => {
		impl_all!(generic, $resource_type, h3, $impl);

		$impl!($resource_type, "TEMP", h3::STemplateEntityFactory);
		$impl!($resource_type, "ECPB", h3::SExtendedCppEntityBlueprint);

		$impl!($resource_type, "ORES", "ORES-activities", h3::SActivities);
	};

	(generic, $resource_type:ident, $game:ident, $impl:ident) => {
		$impl!($resource_type, "AIBB", $game::SBehaviorTreeInfo);
		$impl!($resource_type, "AIRG", $game::SReasoningGrid);
		$impl!($resource_type, "ASVA", Vec<$game::SPackedAnimSetEntry>);
		$impl!($resource_type, "ATMD", $game::ZAMDTake);
		$impl!($resource_type, "BMSK", Vec<u32>);
		$impl!($resource_type, "CBLU", $game::SCppEntityBlueprint);
		$impl!($resource_type, "CPPT", $game::SCppEntity);
		$impl!($resource_type, "CRMD", $game::SCrowdMapData);
		$impl!($resource_type, "ENUM", $game::SEnumType);
		$impl!($resource_type, "GFXF", $game::SGFxMovieResource);
		$impl!($resource_type, "GIDX", $game::SResourceIndex);
		$impl!($resource_type, "TBLU", $game::STemplateEntityBlueprint);
		$impl!($resource_type, "UICB", $game::SControlTypeInfo);
		$impl!($resource_type, "VIDB", $game::SVideoDatabaseData);
		$impl!($resource_type, "WSGB", $game::SAudioStateGroupData);
		$impl!($resource_type, "WSWB", $game::SAudioSwitchGroupData);

		$impl!(
			$resource_type,
			"ORES",
			"ORES-blobs",
			Vec<$game::SBlobsConfigResourceEntry>
		);
		$impl!(
			$resource_type,
			"ORES",
			"ORES-contracts",
			Vec<$game::SContractConfigResourceEntry>
		);
		$impl!($resource_type, "ORES", "ORES-unlockables", EcoString);
		$impl!(
			$resource_type,
			"ORES",
			"ORES-environment",
			$game::SEnvironmentConfigResource
		);
	};
}

fn main() {
	match env::args().nth(1).as_deref() {
		#[cfg(feature = "h1")]
		Some("HM2016") => match env::args().nth(2).as_deref() {
			Some("convert") => {
				let Some(resource_type) = env::args().nth(3) else {
					panic!("3rd argument (resource type) missing or unsupported");
				};

				impl_all!(resource_type, h1, impl_convert);
			}

			Some("generate") => {
				let Some(resource_type) = env::args().nth(3) else {
					panic!("3rd argument (resource type) missing or unsupported");
				};

				impl_all!(resource_type, h1, impl_generate);
			}

			_ => panic!("2nd argument must be convert or generate")
		},

		#[cfg(feature = "h2")]
		Some("HM2") => match env::args().nth(2).as_deref() {
			Some("convert") => {
				let Some(resource_type) = env::args().nth(3) else {
					panic!("3rd argument (resource type) missing or unsupported");
				};

				impl_all!(resource_type, h2, impl_convert);
			}

			Some("generate") => {
				let Some(resource_type) = env::args().nth(3) else {
					panic!("3rd argument (resource type) missing or unsupported");
				};

				impl_all!(resource_type, h2, impl_generate);
			}

			_ => panic!("2nd argument must be convert or generate")
		},

		#[cfg(feature = "h3")]
		Some("HM3") => match env::args().nth(2).as_deref() {
			Some("convert") => {
				let Some(resource_type) = env::args().nth(3) else {
					panic!("3rd argument (resource type) missing or unsupported");
				};

				impl_all!(resource_type, h3, impl_convert);
			}

			Some("generate") => {
				let Some(resource_type) = env::args().nth(3) else {
					panic!("3rd argument (resource type) missing or unsupported");
				};

				impl_all!(resource_type, h3, impl_generate);
			}

			_ => panic!("2nd argument must be convert or generate")
		},

		_ => panic!("1st argument (game) missing or unsupported")
	}
}
