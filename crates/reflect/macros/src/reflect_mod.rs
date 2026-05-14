use quote::quote;

use crate::{reflect_meta, reflect_visibility};

pub fn attr(meta: proc_macro2::TokenStream, item: &mut syn::ItemMod) -> proc_macro2::TokenStream {
    if item.content.is_some() {
        let value = build(meta, item);
        item.content
            .as_mut()
            .unwrap()
            .1
            .push(syn::Item::Verbatim(quote! {
                pub fn type_of() -> ::reflect::Type {
                    return #value;
                }
            }));
    }

    quote!(#item)
}

pub fn build(meta: proc_macro2::TokenStream, item: &mut syn::ItemMod) -> proc_macro2::TokenStream {
    let vis = reflect_visibility::build(&item.vis);
    let inner_meta = reflect_meta::build(&item.attrs);
    let mut children = vec![];

    if let Some((_, items)) = &mut item.content {
        for item in items.iter_mut() {
            match super::reflect_item(item) {
                None => continue,
                Some(v) => children.push(v),
            };
        }
    }

    quote! {
        ::reflect::ModType::new()
            .path(::reflect::Path::from(module_path!()))
            .meta(#meta.merge(&#inner_meta))
            .visibility(#vis)
            .items([#(#children,)*])
            .build()
            .to_type()
    }
}
