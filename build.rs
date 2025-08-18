#![feature(trim_prefix_suffix)]

use std::{collections::VecDeque, fs};

use anyhow::Result;
use codegen::{Block, Scope};
use inflector::Inflector;
use lazy_regex::{regex_captures, regex_is_match, regex_replace};

pub enum Member {
	Padding(usize),
	Field(String, String, String)
}

fn parse_enums(enums: &str) -> Vec<(String, usize, Vec<(String, i64)>)> {
	let enums = enums.replace('\r', "");
	let enums = enums.split_once("#pragma pack(push, 1)\n\n").unwrap().1;
	enums
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
				let (name, members) = section.split_once("\n{").unwrap();

				let name = name.trim_start_matches("enum class ");

				let members = members
					.trim()
					.lines()
					.map(|x| x.trim())
					.map(|member| {
						let (name, value) = member.split_once(" = ").unwrap();
						(name.to_owned(), value.trim_end_matches(",").parse().unwrap())
					})
					.collect();

				Some((name.to_owned(), size, members))
			} else {
				None
			}
		})
		.collect()
}

fn parse_classes(classes: &str) -> Vec<(String, Vec<Member>)> {
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

				Some((name.to_owned(), members))
			} else {
				None
			}
		})
		.collect()
}

fn generate(scope: &mut Scope, code: &str) {
	scope.import("crate::ser", "Aligned");
	scope.import("crate::ser", "Bin1Serialize");
	scope.import("crate::ser", "Bin1Serializer");
	scope.import("crate::ser", "SerializeError");
	scope.import("crate::types::variant", "StaticVariant");
	scope.import("crate::types::variant", "Variant");
	scope.import("crate::types::variant", "VariantDeserializer");
	scope.import("crate::types::variant", "DeserializeVariant");
	scope.raw("use crate as hitman_bin1;");

	let mut classes = parse_classes(code);

	let mut class_queue = VecDeque::new();
	class_queue.push_back(classes.remove(classes.iter().position(|x| x.0 == "STemplateEntityFactory").unwrap()));
	class_queue.push_back(classes.remove(classes.iter().position(|x| x.0 == "STemplateEntityBlueprint").unwrap()));
	class_queue.push_back(classes.remove(classes.iter().position(|x| x.0 == "SEntityTemplateReference").unwrap()));
	class_queue.push_back(classes.remove(classes.iter().position(|x| x.0 == "SColorRGB").unwrap()));
	class_queue.push_back(classes.remove(classes.iter().position(|x| x.0 == "SColorRGBA").unwrap()));
	class_queue.push_back(classes.remove(classes.iter().position(|x| x.0 == "ZGameTime").unwrap()));

	while let Some((name, members)) = class_queue.pop_front() {
		for member in &members {
			if let Member::Field(_, _, ty) = member {
				if ty == "EcoString" {
					scope.import("ecow", "EcoString");
				} else if ty == "ZVariant" {
					scope.import("crate::types::variant", "ZVariant");
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
			.derive("serde::Serialize")
			.derive("serde::Deserialize")
			.vis("pub");

		let mut padding = 0;

		for member in members {
			match member {
				Member::Padding(amount) => {
					padding = amount;
				}

				Member::Field(orig_name, field_name, type_name) => {
					let field = cls
						.new_field(field_name, type_name)
						.vis("pub")
						.annotation(format!("#[serde(rename = \"{}\")]", orig_name));

					if padding != 0 {
						field.annotation(format!("#[bin1(pad = {padding})]"));
					}

					padding = 0;
				}
			}
		}

		scope.new_impl(&name).impl_trait("StaticVariant").associate_const(
			"TYPE_ID",
			"&str",
			format!(r#""{name}""#),
			""
		);

		scope
			.new_impl(&format!("Vec<{name}>"))
			.impl_trait("StaticVariant")
			.associate_const("TYPE_ID", "&str", format!(r#""TArray<{name}>""#), "");

		scope
			.new_impl(&format!("Vec<Vec<{name}>>"))
			.impl_trait("StaticVariant")
			.associate_const("TYPE_ID", "&str", format!(r#""TArray<TArray<{name}>>""#), "");

		let variant_impl = scope.new_impl(&name).impl_trait("Variant");

		variant_impl
			.new_fn("type_id")
			.arg_ref_self()
			.arg(
				"interner",
				"&mut string_interner::StringInterner<string_interner::backend::BucketBackend>"
			)
			.ret("string_interner::DefaultSymbol")
			.line(format!(r#"interner.get_or_intern_static("{name}")"#));

		variant_impl
			.new_fn("to_serde")
			.arg_ref_self()
			.ret("Result<serde_json::Value, serde_json::Error>")
			.line("serde_json::to_value(self)");

		scope.raw(format!(
			"inventory::submit!(&VariantDeserializer::<{name}>::new() as &dyn DeserializeVariant);"
		));
		scope.raw(format!(
			"inventory::submit!(&VariantDeserializer::<Vec<{name}>>::new() as &dyn DeserializeVariant);"
		));
		scope.raw(format!(
			"inventory::submit!(&VariantDeserializer::<Vec<Vec<{name}>>>::new() as &dyn DeserializeVariant);"
		));
	}

	for (name, size, members) in parse_enums(code) {
		if members.is_empty() {
			continue;
		}

		let item = scope
			.new_enum(&name)
			.derive("Debug")
			.derive("Clone")
			.derive("PartialEq")
			.derive("serde::Serialize")
			.derive("serde::Deserialize")
			.vis("pub");

		for (variant_name, _) in &members {
			item.new_variant(variant_name);
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
			.push_block({
				let mut block = Block::new("ser.write_unaligned(&match self");
				for (variant_name, variant_value) in &members {
					block.line(format!("Self::{variant_name} => {variant_value}{signed_size_ty},"));
				}
				block.after(".to_le_bytes());");
				block
			})
			.line("Ok(())");

		scope.new_impl(&name).impl_trait("StaticVariant").associate_const(
			"TYPE_ID",
			"&str",
			format!(r#""{name}""#),
			""
		);

		scope
			.new_impl(&format!("Vec<{name}>"))
			.impl_trait("StaticVariant")
			.associate_const("TYPE_ID", "&str", format!(r#""TArray<{name}>""#), "");

		scope
			.new_impl(&format!("Vec<Vec<{name}>>"))
			.impl_trait("StaticVariant")
			.associate_const("TYPE_ID", "&str", format!(r#""TArray<TArray<{name}>>""#), "");

		let variant_impl = scope.new_impl(&name).impl_trait("Variant");

		variant_impl
			.new_fn("type_id")
			.arg_ref_self()
			.arg(
				"interner",
				"&mut string_interner::StringInterner<string_interner::backend::BucketBackend>"
			)
			.ret("string_interner::DefaultSymbol")
			.line(format!(r#"interner.get_or_intern_static("{name}")"#));

		variant_impl
			.new_fn("to_serde")
			.arg_ref_self()
			.ret("Result<serde_json::Value, serde_json::Error>")
			.line(format!(r#"Ok(serde_json::Value::String("{name}".into()))"#));

		scope.raw(format!(
			"inventory::submit!(&VariantDeserializer::<{name}>::new() as &dyn DeserializeVariant);"
		));
		scope.raw(format!(
			"inventory::submit!(&VariantDeserializer::<Vec<{name}>>::new() as &dyn DeserializeVariant);"
		));
		scope.raw(format!(
			"inventory::submit!(&VariantDeserializer::<Vec<Vec<{name}>>>::new() as &dyn DeserializeVariant);"
		));
	}
}

pub fn main() -> Result<()> {
	let mut h3 = Scope::new();

	generate(&mut h3, &fs::read_to_string("h3.txt")?);

	fs::write(
		"src/generated/h3.rs",
		format!("#![allow(non_camel_case_types, non_snake_case)]\n\n{}", h3.to_string(),)
	)?;

	fs::write("src/generated/mod.rs", r"pub mod h3;")?;

	Ok(())
}
