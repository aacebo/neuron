use quote::{format_ident, quote};

use crate::{reflect_field, reflect_generics, reflect_meta, reflect_visibility};

pub fn derive(input: &syn::DeriveInput, data: &syn::DataEnum) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let ty = build(&syn::ItemEnum {
        attrs: input.attrs.clone(),
        variants: data.variants.clone(),
        generics: input.generics.clone(),
        ident: input.ident.clone(),
        brace_token: data.brace_token,
        enum_token: data.enum_token,
        vis: input.vis.clone(),
    });

    let variants = data
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            let variant_fields = match &variant.fields {
                syn::Fields::Unit => vec![],
                syn::Fields::Named(fields) => fields
                    .named
                    .iter()
                    .map(|field| {
                        let field_ident = &field.ident;
                        quote!(#field_ident)
                    })
                    .collect::<Vec<_>>(),
                syn::Fields::Unnamed(fields) => fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let field_ident = format_ident!("p{}", i);
                        quote!(#field_ident)
                    })
                    .collect::<Vec<_>>(),
            };

            if variant_fields.is_empty() {
                return quote! {
                    Self::#variant_ident => ::reflect::Value::Null
                };
            }

            if variant_fields.len() == 1 {
                return quote! {
                    Self::#variant_ident(v) => ::reflect::ToValue::to_value(v)
                };
            }

            quote! {
                Self::#variant_ident(#(#variant_fields,)*) => {
                    ::reflect::value_of!((#(#variant_fields.clone(),)*))
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        impl ::reflect::TypeOf for #name {
            fn type_of() -> ::reflect::Type {
                ::std::thread_local! {
                    static CACHED: ::std::cell::RefCell<::std::option::Option<::reflect::Type>>
                        = ::std::cell::RefCell::new(::std::option::Option::None);
                }

                CACHED.with(|c| {
                    let mut guard = c.borrow_mut();

                    if guard.is_none() {
                        *guard = ::std::option::Option::Some(#ty);
                    }

                    guard.as_ref().unwrap().clone()
                })
            }
        }

        impl ::reflect::ToType for #name {
            fn to_type(&self) -> ::reflect::Type {
                <Self as ::reflect::TypeOf>::type_of()
            }
        }

        impl ::reflect::Object for #name {
            fn field(&self, name: &::reflect::FieldName) -> ::reflect::Value<'_> {
                match self {
                    #(#variants,)*
                }
            }
        }

        impl ::reflect::ToValue for #name {
            fn to_value(&self) -> ::reflect::Value<'_> {
                ::reflect::Value::Dynamic(::reflect::Dynamic::from_object(self))
            }
        }

    }
}

pub fn attr(item: &syn::ItemEnum) -> proc_macro2::TokenStream {
    let name = &item.ident;
    let ty = build(item);
    let variants = item
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            let variant_fields = match &variant.fields {
                syn::Fields::Unit => vec![],
                syn::Fields::Named(fields) => fields
                    .named
                    .iter()
                    .map(|field| {
                        let field_ident = &field.ident;
                        quote!(#field_ident)
                    })
                    .collect::<Vec<_>>(),
                syn::Fields::Unnamed(fields) => fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let field_ident = format_ident!("p{}", i);
                        quote!(#field_ident)
                    })
                    .collect::<Vec<_>>(),
            };

            if variant_fields.is_empty() {
                return quote! {
                    Self::#variant_ident => ::reflect::Value::Null
                };
            }

            if variant_fields.len() == 1 {
                return quote! {
                    Self::#variant_ident(v) => ::reflect::ToValue::to_value(v)
                };
            }

            quote! {
                Self::#variant_ident(#(#variant_fields,)*) => {
                    ::reflect::value_of!((#(#variant_fields.clone(),)*))
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        impl ::reflect::TypeOf for #name {
            fn type_of() -> ::reflect::Type {
                ::std::thread_local! {
                    static CACHED: ::std::cell::RefCell<::std::option::Option<::reflect::Type>>
                        = ::std::cell::RefCell::new(::std::option::Option::None);
                }

                CACHED.with(|c| {
                    let mut guard = c.borrow_mut();

                    if guard.is_none() {
                        *guard = ::std::option::Option::Some(#ty);
                    }

                    guard.as_ref().unwrap().clone()
                })
            }
        }

        impl ::reflect::ToType for #name {
            fn to_type(&self) -> ::reflect::Type {
                <Self as ::reflect::TypeOf>::type_of()
            }
        }

        impl ::reflect::ToValue for #name {
            fn to_value(&self) -> ::reflect::Value<'_> {
                match self {
                    #(#variants,)*
                }
            }
        }

    }
}

pub fn build(item: &syn::ItemEnum) -> proc_macro2::TokenStream {
    let name = &item.ident;
    let vis = reflect_visibility::build(&item.vis);
    let meta = reflect_meta::build(&item.attrs);
    let generics = reflect_generics::build(&item.generics);
    let variants = item
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let variant_meta = reflect_meta::build(&variant.attrs);

            match &variant.fields {
                syn::Fields::Unit => quote! {
                    ::reflect::Variant::new()
                        .name(stringify!(#variant_name))
                        .build()
                },
                syn::Fields::Named(named_fields) => {
                    let fields = named_fields
                        .named
                        .iter()
                        .enumerate()
                        .map(|(i, field)| reflect_field::build(field, i, true))
                        .collect::<Vec<_>>();

                    quote! {
                        ::reflect::Variant::new()
                            .name(stringify!(#variant_name))
                            .meta(#variant_meta)
                            .fields(
                                ::reflect::Fields::new()
                                    .layout(::reflect::Layout::Key)
                                    .fields([#(#fields,)*])
                                    .build()
                            )
                            .build()
                    }
                }
                syn::Fields::Unnamed(unnamed_fields) => {
                    let fields = unnamed_fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(i, field)| reflect_field::build(field, i, false))
                        .collect::<Vec<_>>();

                    quote! {
                        ::reflect::Variant::new()
                            .name(stringify!(#variant_name))
                            .meta(#variant_meta)
                            .fields(
                                ::reflect::Fields::new()
                                    .layout(::reflect::Layout::Index)
                                    .fields([#(#fields,)*])
                                    .build()
                            )
                            .build()
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        ::reflect::EnumType::new()
            .path(::reflect::Path::from(module_path!()))
            .name(stringify!(#name))
            .meta(#meta)
            .generics(#generics)
            .visibility(#vis)
            .variants([#(#variants,)*])
            .build()
            .to_type()
    }
}
