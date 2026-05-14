use quote::quote;

use crate::parse;

pub fn build(attributes: &[syn::Attribute]) -> proc_macro2::TokenStream {
    let mut pairs = vec![];

    for attr in attributes.iter().filter(|a| a.path().is_ident("reflect")) {
        let _ = attr.parse_nested_meta(|meta| {
            pairs.push(parse::meta_data_item(meta));
            Ok(())
        });
    }

    if pairs.is_empty() {
        return quote!(::reflect::MetaData::new());
    }

    quote! {
        ::reflect::MetaData::from([#(#pairs,)*])
    }
}
