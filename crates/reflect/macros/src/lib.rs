mod parse;
mod reflect_enum;
mod reflect_field;
mod reflect_generics;
mod reflect_impl;
mod reflect_meta;
mod reflect_mod;
mod reflect_struct;
mod reflect_trait;
mod reflect_visibility;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;

#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_reflect(tokens: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(tokens as syn::DeriveInput);

    match &input.data {
        syn::Data::Struct(ty) => reflect_struct::derive(&input, ty),
        syn::Data::Enum(ty) => reflect_enum::derive(&input, ty),
        _ => quote::quote!(compile_error!("unsupported Reflect type")),
    }
    .into()
}

#[proc_macro_attribute]
pub fn reflect(attrs: TokenStream, item_tokens: TokenStream) -> TokenStream {
    let mut item = syn::parse_macro_input!(item_tokens as syn::Item);

    match reflect_attr(attrs, &mut item) {
        None => quote!(compile_error!("invalid reflect type")),
        Some(v) => v,
    }
    .into()
}

fn reflect_attr(attrs: TokenStream, item: &mut syn::Item) -> Option<proc_macro2::TokenStream> {
    let mut pairs = vec![];
    let parser = syn::meta::parser(|meta| {
        pairs.push(parse::meta_data_item(meta));
        Ok(())
    });

    if let Err(err) = &parser.parse(attrs) {
        return err.to_compile_error().into();
    }

    let meta = if pairs.is_empty() {
        quote!(::reflect::MetaData::new())
    } else {
        quote!(::reflect::MetaData::from([#(#pairs,)*]))
    };

    match item {
        syn::Item::Mod(v) => Some(reflect_mod::attr(meta, v)),
        syn::Item::Trait(v) => Some(reflect_trait::attr(meta, v)),
        syn::Item::Struct(v) => Some(reflect_struct::attr(v)),
        syn::Item::Enum(v) => Some(reflect_enum::attr(v)),
        _ => None,
    }
}

fn reflect_ty(item: &mut syn::Item) -> Option<proc_macro2::TokenStream> {
    match item {
        syn::Item::Mod(v) => Some(reflect_mod::build(quote!(), v)),
        syn::Item::Trait(v) => Some(reflect_trait::build(quote!(), v)),
        syn::Item::Struct(v) => Some(reflect_struct::build(v)),
        syn::Item::Enum(v) => Some(reflect_enum::build(v)),
        _ => None,
    }
}

fn reflect_item(item: &mut syn::Item) -> Option<proc_macro2::TokenStream> {
    let value = match item {
        syn::Item::Impl(v) => Some(reflect_impl::build(v)),
        other => reflect_ty(other),
    };

    value.as_ref().map(|value| quote!(#value.to_item()))
}
