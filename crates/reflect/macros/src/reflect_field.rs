use quote::quote;

use crate::{reflect_meta, reflect_visibility};

pub fn build(field: &syn::Field, index: usize, is_named: bool) -> proc_macro2::TokenStream {
    let field_ident = &field.ident;
    let field_name = if is_named {
        quote!(::reflect::FieldName::from(stringify!(#field_ident)))
    } else {
        quote!(::reflect::FieldName::from(#index))
    };

    let field_type = &field.ty;
    let field_vis = reflect_visibility::build(&field.vis);
    let field_meta = reflect_meta::build(&field.attrs);

    quote! {
        ::reflect::Field::new()
            .name(#field_name)
            .ty(::reflect::type_of!(#field_type))
            .visibility(#field_vis)
            .meta(#field_meta)
            .build()
    }
}
