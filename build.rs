#![feature(trim_prefix_suffix)]

use std::{
	collections::{HashMap, VecDeque},
	env, fs,
	path::PathBuf
};

use anyhow::Result;
use codegen::{Block, Scope};
use edit_distance::edit_distance;
use inflector::Inflector;
use lazy_regex::{regex_captures, regex_captures_iter, regex_is_match, regex_replace};
use rayon::prelude::*;

pub enum Member {
	Padding(usize),
	Field(String, String, String)
}

fn parse_enums(classes: &str, enums: &str) -> Vec<(String, String, usize, Vec<(String, i64)>)> {
	let enums = enums.replace('\r', "");
	let enums = regex_captures_iter!(
		r#"\(\*g_Enums\)\["(.*?)"] = \{\n((?:\s+\{ -?\d+, ".*?" \},\n)*)\s+\};"#,
		&enums
	)
	.collect::<Vec<_>>();

	let classes = classes.replace('\r', "");
	let classes = classes.split_once("#pragma pack(push, 1)\n\n").unwrap().1;
	classes
		.split("};\n\n")
		.filter(|section| section.starts_with("// Size:"))
		.filter_map(|section| {
			let size =
				usize::from_str_radix(regex_captures!(r"// Size: 0x([0-9A-F]+)", section).unwrap().1, 16).unwrap();

			let section = section
				.split('\n')
				.filter(|x| !x.is_empty() && !x.trim_start().starts_with("//"))
				.collect::<Vec<_>>()
				.join("\n");

			if section.starts_with("enum class") {
				let (name, _) = section.split_once("\n{").unwrap();

				let name = name.trim_start_matches("enum class ").trim();

				let enum_entry = enums.par_iter().min_by_key(|x| edit_distance(name, &x[1])).unwrap();

				let members = enum_entry[2]
					.lines()
					.map(|x| regex_captures!(r#"\{\s*(-?\d+),\s*"(.+?)"\s*\}"#, x))
					.filter_map(|x| x.map(|x| (x.2.to_owned(), x.1.parse::<i64>().unwrap())))
					.collect::<Vec<_>>();

				Some((name.to_owned(), enum_entry[1].to_owned(), size, members))
			} else {
				None
			}
		})
		.collect()
}

fn parse_classes(classes: &str, types: &str) -> Vec<(String, String, Vec<Member>)> {
	let types = types
		.lines()
		.filter(|x| x.trim().starts_with("ZHMTypeInfo "))
		.map(|x| regex_captures!(r#" (.*?)::TypeInfo = ZHMTypeInfo\("(.*)?","#, x).unwrap())
		.map(|(_, x, y)| (x, y))
		.collect::<HashMap<_, _>>();

	let classes = classes.replace('\r', "");
	let classes = classes.split_once("#pragma pack(push, 1)\n\n").unwrap().1;
	classes
		.split("};\n\n")
		.filter(|section| section.starts_with("// Size:"))
		.filter_map(|section| {
			let section = section
				.split('\n')
				.filter(|x| !x.is_empty() && !x.trim_start().starts_with("//"))
				.collect::<Vec<_>>()
				.join("\n");

			if section.starts_with("class") {
				let (name, members) = section.split_once("\n{\npublic:\n").unwrap();

				let name = name.trim_start_matches("class ");
				let name = regex_replace!(r" */\*.*?\*/ *", name, "").into_owned();

				let members = members
					.lines()
					.map(|x| x.trim())
					.filter(|x| !x.is_empty() && !x.starts_with("static") && !x.starts_with("bool operator"))
					.map(|member| {
						if let Some((_, amount)) = regex_captures!(r"uint8_t _pad\w+\[(\d+)\] \{\};", member) {
							Member::Padding(amount.parse().unwrap())
						} else {
							let (_, type_name, field_name) = regex_captures!(r"^(.+) (.+);.*$", member).unwrap();

							let original_field_name = field_name;

							let field_name = if field_name.len() != 2 && !regex_is_match!(r"m\d+", field_name) {
								field_name.to_snake_case()
							} else {
								field_name.into()
							};

							let field_name = if let Some((start, rest)) = field_name.split_once('_')
								&& start.len() == 1 && !["x", "y", "z"].contains(&start)
								&& !rest.is_empty()
							{
								rest.into()
							} else {
								field_name
							};

							let field_name = if let Some((start, rest)) = field_name.split_once('_')
								&& start.len() == 1 && !["x", "y", "z"].contains(&start)
								&& !rest.is_empty()
							{
								rest.into()
							} else {
								field_name
							};

							let field_name = match field_name.as_str() {
								"type" => "r#type",
								"ref" => "reference",
								"move" => "r#move",
								x => x
							};

							fn process_type_name(type_name: &str) -> String {
								match type_name {
									"int8_t" => "i8".into(),
									"int16" => "i16".into(),
									"int32" => "i32".into(),
									"int64" => "i64".into(),

									"char" => "u8".into(),
									"uint8" => "u8".into(),
									"uint8_t" => "u8".into(),
									"uint16" => "u16".into(),
									"uint32" => "u32".into(),
									"uint64" => "u64".into(),

									"float32" => "f32".into(),
									"float64" => "f64".into(),

									"bool" => "bool".into(),

									"ZString" => "EcoString".into(),

									x if x.starts_with("TArray<") => format!(
										"Vec<{}>",
										process_type_name(
											&x.trim_prefix("TArray<")
												.chars()
												.rev()
												.skip(1)
												.collect::<Vec<_>>()
												.into_iter()
												.rev()
												.collect::<String>()
										)
									),

									x if x.starts_with("TFixedArray<") => format!(
										"[{}; {}]",
										process_type_name(regex_captures!(r"TFixedArray<(.*), *(.*)>", x).unwrap().1),
										regex_captures!(r"TFixedArray<(.*), *(.*)>", x)
											.unwrap()
											.2
											.parse::<usize>()
											.unwrap()
									),

									x if x.starts_with("TPair<") => format!(
										"({}, {})",
										process_type_name(regex_captures!(r"TPair<(.*), *(.*)>", x).unwrap().1),
										process_type_name(regex_captures!(r"TPair<(.*), *(.*)>", x).unwrap().2)
									),

									x => x.into()
								}
							}

							let type_name = process_type_name(type_name);

							Member::Field(
								original_field_name.to_owned(),
								field_name.to_owned(),
								type_name.to_owned()
							)
						}
					})
					.collect::<Vec<_>>();

				Some((
					name.to_owned(),
					(*types.get(name.as_str()).unwrap()).to_owned(),
					members
				))
			} else {
				None
			}
		})
		.collect()
}

fn generate(scope: &mut Scope, classes_code: &str, enums_code: &str, types_code: &str) {
	scope.import("crate::ser", "Aligned");
	scope.import("crate::ser", "Bin1Serialize");
	scope.import("crate::ser", "Bin1Serializer");
	scope.import("crate::ser", "SerializeError");
	scope.import("crate::de", "Bin1Deserialize");
	scope.import("crate::de", "Bin1Deserializer");
	scope.import("crate::de", "DeserializeError");
	scope.import("crate::types::variant", "StaticVariant");
	scope.import("crate::types::variant", "Variant");
	scope.raw("use crate as hitman_bin1;");

	let mut classes = parse_classes(classes_code, types_code);

	// Special cased
	classes.remove(classes.iter().position(|x| x.0 == "ZRuntimeResourceID").unwrap());
	classes.remove(classes.iter().position(|x| x.0 == "SEntityTemplateProperty").unwrap());

	let mut class_queue = VecDeque::new();

	for ty in [
		"STemplateEntity",
		"STemplateEntityFactory",
		"STemplateEntityBlueprint",
		"SColorRGB",
		"SColorRGBA",
		"ZGuid",
		"ZGameTime",
		"SVector2",
		"SVector3",
		"SVector4",
		"SMatrix43",
		"SWorldSpaceSettings",
		"S25DProjectionSettings",
		"SBodyPartDamageMultipliers",
		"SCCEffectSet",
		"SSCCuriousConfiguration",
		"ZCurve",
		"SMapMarkerData",
		"ZHUDOccluderTriggerEntity_SBoneTestSetup",
		"SGaitTransitionEntry",
		"SClothVertex",
		"ZSharedSensorDef_SVisibilitySetting",
		"SFontLibraryDefinition",
		"SCamBone",
		"SConversationPart",
		"AI_SFirePattern01",
		"STargetableBoneConfiguration",
		"ZSecuritySystemCameraConfiguration_SHitmanVisibleEscalationRule",
		"AI_SFirePattern02",
		"ZSecuritySystemCameraConfiguration_SDeadBodyVisibleEscalationRule"
	] {
		if let Some(pos) = classes.iter().position(|x| x.0 == ty) {
			class_queue.push_back(classes.remove(pos));
		}
	}

	while let Some((name, type_id, members)) = class_queue.pop_front() {
		for member in &members {
			if let Member::Field(_, _, ty) = member {
				if ty == "EcoString" {
					scope.import("ecow", "EcoString");
				} else if ty == "ZRuntimeResourceID" {
					scope.import("crate::types::resource", "ZRuntimeResourceID");
				} else {
					let mut tys = vec![ty.trim_start_matches("Vec<").trim_end_matches(">")];
					for ty in tys.clone() {
						if ty.starts_with('(') {
							let (first, second) = ty
								.trim_start_matches('(')
								.trim_end_matches(')')
								.split_once(',')
								.unwrap();

							tys.push(first.trim());
							tys.push(second.trim());
						}
					}

					for ty in tys {
						if let Some(pos) = classes.iter().position(|x| x.0 == *ty) {
							class_queue.push_back(classes.remove(pos));
						}
					}
				}
			}
		}

		let cls = scope
			.new_struct(&name)
			.derive("Debug")
			.derive("Clone")
			.derive("PartialEq")
			.derive("Bin1Serialize")
			.derive("Bin1Deserialize")
			.derive("serde::Serialize")
			.derive("serde::Deserialize")
			.vis("pub");

		let mut padding = 0;

		let mut last_field = None;

		for member in members {
			match member {
				Member::Padding(amount) => {
					padding = amount;
				}

				Member::Field(orig_name, field_name, type_name) => {
					last_field = Some({
						let field = cls
							.new_field(field_name, type_name)
							.vis("pub")
							.annotation(format!("#[serde(rename = \"{}\")]", orig_name));

						if padding != 0 {
							field.annotation(format!("#[bin1(pad = {padding})]"));
						}

						field
					});

					padding = 0;
				}
			}
		}

		if padding != 0 {
			last_field.unwrap().annotation(format!("#[bin1(pad_end = {padding})]"));
		}

		scope.new_impl(&name).impl_trait("StaticVariant").associate_const(
			"TYPE_ID",
			"&str",
			format!(r#""{type_id}""#),
			""
		);

		scope
			.new_impl(&format!("Vec<{name}>"))
			.impl_trait("StaticVariant")
			.associate_const("TYPE_ID", "&str", format!(r#""TArray<{type_id}>""#), "");

		scope
			.new_impl(&format!("Vec<Vec<{name}>>"))
			.impl_trait("StaticVariant")
			.associate_const("TYPE_ID", "&str", format!(r#""TArray<TArray<{type_id}>>""#), "");

		let variant_impl = scope.new_impl(&name).impl_trait("Variant");

		variant_impl
			.new_fn("type_id")
			.arg_ref_self()
			.arg(
				"interner",
				"&mut string_interner::StringInterner<string_interner::backend::BucketBackend>"
			)
			.ret("string_interner::DefaultSymbol")
			.line(format!(r#"interner.get_or_intern_static("{type_id}")"#));

		variant_impl
			.new_fn("to_serde")
			.arg_ref_self()
			.ret("Result<serde_json::Value, serde_json::Error>")
			.line("serde_json::to_value(self)");

		scope.raw(format!("submit!({name});"));
	}

	for (name, type_id, size, members) in parse_enums(classes_code, enums_code) {
		let item = scope
			.new_enum(&name)
			.derive("Debug")
			.derive("Clone")
			.derive("Copy")
			.derive("PartialEq")
			.derive("serde::Serialize")
			.derive("serde::Deserialize")
			.vis("pub");

		if members.is_empty() {
			// ZST
			item.new_variant("Value").annotation(r#"#[serde(rename = "")]"#);
		} else {
			for (variant_name, _) in &members {
				item.new_variant(variant_name);
			}
		}

		let size_ty = match size {
			1 => "u8",
			2 => "u16",
			4 => "u32",
			8 => "u64",
			_ => panic!("Invalid size")
		};

		let signed_size_ty = match size {
			1 => "i8",
			2 => "i16",
			4 => "i32",
			8 => "i64",
			_ => panic!("Invalid size")
		};

		item.repr(size_ty);

		scope
			.new_impl(&name)
			.impl_trait("Aligned")
			.associate_const("ALIGNMENT", "usize", size.to_string(), "");

		scope
			.new_impl(signed_size_ty)
			.impl_trait(format!("From<{name}>"))
			.new_fn("from")
			.arg("value", &name)
			.ret(signed_size_ty)
			.push_block({
				let mut block = Block::new("match value");
				if members.is_empty() {
					block.line(format!("{name}::Value => 1"));
				} else {
					for (variant_name, variant_value) in &members {
						block.line(format!("{name}::{variant_name} => {variant_value},"));
					}
				}
				block
			});

		scope
			.new_impl(&name)
			.impl_trait(format!("TryFrom<{signed_size_ty}>"))
			.associate_type("Error", "()")
			.new_fn("try_from")
			.arg("value", signed_size_ty)
			.ret(format!("Result<{name}, ()>"))
			.push_block({
				if members.is_empty() {
					let mut block = Block::new("");
					block.line(format!(
						r#"if value != 1 {{ eprintln!("Unexpected value for uninhabited enum {type_id}: {{}}", value); }}"#
					));
					block.line("Ok(Self::Value)");
					block
				} else {
					let mut block = Block::new("Ok(match value");
					for (variant_name, variant_value) in &members {
						block.line(format!("{variant_value} => Self::{variant_name},"));
					}
					block.line("_ => return Err(())");
					block.after(")");
					block
				}
			});

		let ser_impl = scope.new_impl(&name).impl_trait("Bin1Serialize");
		ser_impl
			.new_fn("alignment")
			.arg_ref_self()
			.ret("usize")
			.line(size.to_string());
		ser_impl
			.new_fn("write")
			.arg_ref_self()
			.arg("ser", "&mut Bin1Serializer")
			.ret("Result<(), SerializeError>")
			.line(format!(
				"ser.write_unaligned(&{signed_size_ty}::from(*self).to_le_bytes());"
			))
			.line("Ok(())");

		let de_impl = scope.new_impl(&name).impl_trait("Bin1Deserialize");
		de_impl.associate_const("SIZE", "usize", size.to_string(), "");
		de_impl
			.new_fn("read")
			.arg("de", "&mut Bin1Deserializer")
			.ret(format!("Result<{name}, DeserializeError>"))
			.line(format!(
				r"let value = de.read_{signed_size_ty}()?;
value.try_into().map_err(|_| DeserializeError::InvalidEnumValue(value as i64))"
			));

		scope.new_impl(&name).impl_trait("StaticVariant").associate_const(
			"TYPE_ID",
			"&str",
			format!(r#""{type_id}""#),
			""
		);

		scope
			.new_impl(&format!("Vec<{name}>"))
			.impl_trait("StaticVariant")
			.associate_const("TYPE_ID", "&str", format!(r#""TArray<{type_id}>""#), "");

		scope
			.new_impl(&format!("Vec<Vec<{name}>>"))
			.impl_trait("StaticVariant")
			.associate_const("TYPE_ID", "&str", format!(r#""TArray<TArray<{type_id}>>""#), "");

		let variant_impl = scope.new_impl(&name).impl_trait("Variant");

		variant_impl
			.new_fn("type_id")
			.arg_ref_self()
			.arg(
				"interner",
				"&mut string_interner::StringInterner<string_interner::backend::BucketBackend>"
			)
			.ret("string_interner::DefaultSymbol")
			.line(format!(r#"interner.get_or_intern_static("{type_id}")"#));

		variant_impl
			.new_fn("to_serde")
			.arg_ref_self()
			.ret("Result<serde_json::Value, serde_json::Error>")
			.line("serde_json::to_value(self)");

		scope.raw(format!("submit!({name});"));
	}
}

pub fn main() -> Result<()> {
	let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

	fs::write(
		out_dir.join("properties-crc32.txt"),
		fs::read_to_string("properties.txt")?
			.lines()
			.map(|x| crc32fast::hash(x.trim().as_bytes()).to_string())
			.collect::<Vec<_>>()
			.join("\n")
	)?;

	let mut h1 = Scope::new();

	generate(
		&mut h1,
		&fs::read_to_string("h1.txt")?,
		&fs::read_to_string("h1-enums.txt")?,
		&fs::read_to_string("h1-types.txt")?
	);

	fs::write(out_dir.join("h1.rs"), h1.to_string())?;

	let mut h2 = Scope::new();

	generate(
		&mut h2,
		&fs::read_to_string("h2.txt")?,
		&fs::read_to_string("h2-enums.txt")?,
		&fs::read_to_string("h2-types.txt")?
	);

	fs::write(out_dir.join("h2.rs"), h2.to_string())?;

	let mut h3 = Scope::new();

	generate(
		&mut h3,
		&fs::read_to_string("h3.txt")?,
		&fs::read_to_string("h3-enums.txt")?,
		&fs::read_to_string("h3-types.txt")?
	);

	fs::write(out_dir.join("h3.rs"), h3.to_string())?;

	println!("cargo::rerun-if-changed=build.rs");
	println!("cargo::rerun-if-changed=h1.txt");
	println!("cargo::rerun-if-changed=h1-enums.txt");
	println!("cargo::rerun-if-changed=h1-types.txt");
	println!("cargo::rerun-if-changed=h2.txt");
	println!("cargo::rerun-if-changed=h2-enums.txt");
	println!("cargo::rerun-if-changed=h2-types.txt");
	println!("cargo::rerun-if-changed=h3.txt");
	println!("cargo::rerun-if-changed=h3-enums.txt");
	println!("cargo::rerun-if-changed=h3-types.txt");
	println!("cargo::rerun-if-changed=properties.txt");

	Ok(())
}
