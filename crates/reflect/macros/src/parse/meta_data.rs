use quote::quote;

pub fn meta_data_item(meta: syn::meta::ParseNestedMeta<'_>) -> proc_macro2::TokenStream {
    let key_ident = match meta.path.require_ident() {
        Ok(v) => v,
        Err(err) => return err.to_compile_error(),
    };

    let key_str = key_ident.to_string();

    // If there is no "= …", treat as `key = "true"`
    if meta.input.is_empty() {
        let key_lit = syn::LitStr::new(&key_str, key_ident.span());
        return quote!((#key_lit, ::reflect::Value::Null));
    }

    // Expect `= "value"`
    if let Err(err) = meta.input.parse::<syn::Token![=]>() {
        return err.to_compile_error();
    }

    let val: syn::Lit = match meta.input.parse() {
        Ok(v) => v,
        Err(err) => return err.to_compile_error(),
    };

    let key_lit = syn::LitStr::new(&key_str, key_ident.span());
    quote!((#key_lit, ::reflect::value_of!(#val)))
}
