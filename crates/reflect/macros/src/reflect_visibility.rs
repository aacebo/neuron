use quote::{ToTokens, quote};

pub fn build(vis: &syn::Visibility) -> proc_macro2::TokenStream {
    match vis {
        syn::Visibility::Inherited => quote!(::reflect::Visibility::Private),
        syn::Visibility::Public(_) => quote! {
            ::reflect::Visibility::Public(
                ::reflect::Public::Full
            )
        },
        syn::Visibility::Restricted(v) => {
            let path = v.path.to_token_stream().to_string();

            match path.as_str() {
                "self" => quote! {
                    ::reflect::Visibility::Public(
                        ::reflect::Public::Type
                    )
                },
                "crate" => quote! {
                    ::reflect::Visibility::Public(
                        ::reflect::Public::Crate
                    )
                },
                "super" => quote! {
                    ::reflect::Visibility::Public(
                        ::reflect::Public::Super
                    )
                },
                other => quote! {
                    ::reflect::Visibility::Public(
                        ::reflect::Public::Mod(#other.to_string())
                    )
                },
            }
        }
    }
}
