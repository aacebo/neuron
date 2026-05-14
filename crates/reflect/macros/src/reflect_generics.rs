use quote::quote;

pub fn build(generics: &syn::Generics) -> proc_macro2::TokenStream {
    let mut params = vec![];

    for param in &generics.params {
        params.push(match param {
            syn::GenericParam::Lifetime(v) => build_lifetime(v),
            syn::GenericParam::Type(v) => build_type(v),
            syn::GenericParam::Const(v) => build_const(v),
        });
    }

    if params.is_empty() {
        return quote!(::reflect::Generics::new());
    }

    quote! {
        ::reflect::Generics::from([#(#params.to_generic(),)*])
    }
}

pub fn build_lifetime(param: &syn::LifetimeParam) -> proc_macro2::TokenStream {
    let name = &param.lifetime.ident;
    let mut bounds = vec![];

    for lifetime in &param.bounds {
        let lifetime_name = &lifetime.ident;

        bounds.push(quote! {
            ::reflect::LifetimeBound::new(stringify!(#lifetime_name))
        });
    }

    quote! {
        ::reflect::LifetimeParam::new(
            stringify!(#name),
            &[#(#bounds,)*],
        )
    }
}

pub fn build_type(param: &syn::TypeParam) -> proc_macro2::TokenStream {
    let name = &param.ident;
    let mut bounds = vec![];

    for ty in &param.bounds {
        bounds.push(build_bound(ty));
    }

    let tokens = quote! {
        ::reflect::TypeParam::new()
            .name(stringify!(#name))
            .bounds([#(#bounds.to_bound(),)*])
    };

    match &param.default {
        None => quote!(#tokens.build()),
        Some(default) => {
            quote!(#tokens.default(::reflect::type_of!(#default)).build())
        }
    }
}

pub fn build_const(param: &syn::ConstParam) -> proc_macro2::TokenStream {
    let name = &param.ident;
    let ty = &param.ty;
    let tokens = quote! {
        ::reflect::ConstParam::new(
            stringify!(#name),
            &(::reflect::type_of!(#ty)),
        )
    };

    match &param.default {
        None => tokens,
        Some(default) => quote!(#tokens.default(#default)),
    }
}

pub fn build_bound(bound: &syn::TypeParamBound) -> proc_macro2::TokenStream {
    match bound {
        syn::TypeParamBound::Lifetime(v) => build_lifetime_bound(v),
        syn::TypeParamBound::Trait(v) => build_trait_bound(v),
        syn::TypeParamBound::Verbatim(v) => v.clone(),
        _ => quote!(),
    }
}

pub fn build_lifetime_bound(bound: &syn::Lifetime) -> proc_macro2::TokenStream {
    let name = &bound.ident;

    quote! {
        ::reflect::LifetimeBound::new(stringify!(#name))
    }
}

pub fn build_trait_bound(bound: &syn::TraitBound) -> proc_macro2::TokenStream {
    let path = &bound.path;
    let modifier = match &bound.modifier {
        syn::TraitBoundModifier::None => quote!(::reflect::TraitBoundModifier::None),
        syn::TraitBoundModifier::Maybe(_) => quote!(::reflect::TraitBoundModifier::Maybe),
    };

    quote! {
        ::reflect::TraitBound::new(
            &(::reflect::Path::from(#path)),
            #modifier,
        )
    }
}
