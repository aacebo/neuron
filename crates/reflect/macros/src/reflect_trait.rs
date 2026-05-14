use quote::quote;

use crate::{reflect_generics, reflect_meta, reflect_visibility};

pub fn attr(meta: proc_macro2::TokenStream, item: &syn::ItemTrait) -> proc_macro2::TokenStream {
    let name = &item.ident;
    let ty = build(meta, item);

    quote! {
        #item

        impl ::reflect::TypeOf for dyn #name {
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

        impl ::reflect::ToType for dyn #name {
            fn to_type(&self) -> ::reflect::Type {
                <dyn #name as ::reflect::TypeOf>::type_of()
            }
        }
    }
}

pub fn build(meta: proc_macro2::TokenStream, item: &syn::ItemTrait) -> proc_macro2::TokenStream {
    let name = &item.ident;
    let vis = reflect_visibility::build(&item.vis);
    let inner_meta = reflect_meta::build(&item.attrs);
    let generics = reflect_generics::build(&item.generics);
    let methods = item
        .items
        .iter()
        .filter_map(|item| {
            if let syn::TraitItem::Fn(func) = item {
                let fn_name = &func.sig.ident;
                let fn_meta = reflect_meta::build(&func.attrs);
                let fn_is_async = func.sig.asyncness.is_some();

                let fn_return_type = match &func.sig.output {
                    syn::ReturnType::Default => quote!(::reflect::Type::Void),
                    syn::ReturnType::Type(_, ty) => quote!(::reflect::type_of!(#ty)),
                };

                let fn_params = func
                    .sig
                    .inputs
                    .iter()
                    .map(|arg| match arg {
                        syn::FnArg::Receiver(recv) => {
                            let mut param_ty = quote! {
                                ::reflect::ThisType.to_type()
                            };

                            if recv.mutability.is_some() {
                                param_ty = quote!(::reflect::MutType::new(#param_ty).to_type());
                            }

                            if let syn::Type::Reference(_) = recv.ty.as_ref() {
                                param_ty = quote!(::reflect::RefType::new(#param_ty).to_type());
                            }

                            quote! {
                                ::reflect::Param::new(
                                    "self",
                                    #param_ty,
                                )
                            }
                        }
                        syn::FnArg::Typed(typed) => match typed.pat.as_ref() {
                            syn::Pat::Ident(ident) => {
                                let arg_name = &ident.ident;
                                let arg_ty = &typed.ty;
                                let mut param_ty = quote!(::reflect::type_of!(#arg_ty));

                                if ident.mutability.is_some() {
                                    param_ty = quote!(::reflect::MutType::new(#param_ty).to_type());
                                }

                                if let syn::Type::Reference(reference) = typed.ty.as_ref() {
                                    let inner = &reference.elem;
                                    let mut inner_ty = quote!(::reflect::type_of!(#inner));

                                    if reference.mutability.is_some() {
                                        inner_ty = quote!(
                                            ::reflect::MutType::new(#inner_ty).to_type()
                                        );
                                    }

                                    param_ty = quote!(
                                        ::reflect::RefType::new(#inner_ty).to_type()
                                    );
                                }

                                quote! {
                                    ::reflect::Param::new(
                                        stringify!(#arg_name),
                                        #param_ty,
                                    )
                                }
                            }
                            _ => quote!(),
                        },
                    })
                    .collect::<Vec<_>>();

                return Some(quote! {
                    ::reflect::Method::new()
                        .name(stringify!(#fn_name))
                        .meta(#fn_meta)
                        .is_async(#fn_is_async)
                        .visibility(::reflect::Visibility::Public(::reflect::Public::Full))
                        .params([#(#fn_params,)*])
                        .return_type(#fn_return_type)
                        .build()
                });
            }

            None
        })
        .collect::<Vec<_>>();

    quote! {
        ::reflect::TraitType::new()
            .path(::reflect::Path::from(module_path!()))
            .name(stringify!(#name))
            .meta(#meta.merge(&#inner_meta))
            .generics(#generics)
            .visibility(#vis)
            .methods([#(#methods,)*])
            .build()
            .to_type()
    }
}
