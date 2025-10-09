#![allow(non_camel_case_types)]

#[linkme::distributed_slice]
static VARIANT_DESERIALIZERS_H2: [&'static dyn DeserializeVariant];

#[static_init::dynamic]
pub static DESERIALIZERS: HashMap<&'static str, &'static dyn DeserializeVariant> =
	VARIANT_DESERIALIZERS_H2.iter().map(|&x| (x.type_id(), x)).collect();

macro_rules! submit {
	($ty:ty) => {
		mident::mident! {
			#[linkme::distributed_slice(VARIANT_DESERIALIZERS_H2)]
			static #concat(#flatten($ty) _De): &'static dyn DeserializeVariant
				= &VariantDeserializer::<$ty>::new() as &dyn DeserializeVariant;

			#[linkme::distributed_slice(VARIANT_DESERIALIZERS_H2)]
			static #concat(#flatten($ty) _Vec_De): &'static dyn DeserializeVariant
				= &VariantDeserializer::<Vec<$ty>>::new() as &dyn DeserializeVariant;
		}
	};
}

include!("variant_impl.rs");

include!(concat!(env!("OUT_DIR"), "/h2.rs"));
