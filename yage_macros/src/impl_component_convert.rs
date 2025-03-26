use std::cell::Cell;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    Attribute, Generics, Ident, PredicateType, Token, WherePredicate, parse::Parse, token::Where,
};

use crate::utils::Array;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Opt {
    Async,
    Sync,
    Stateless,
    Dynamic,
    None,
}

impl Opt {
    fn bound_for_options(tokens: [Self; 2]) -> Result<WherePredicate, syn::Error> {
        let [syncness, dynamic] = tokens;

        let tokens = match (syncness, dynamic) {
            (Opt::Sync, Opt::None) => quote! {
                Self: ::yage_core::component::Component<State = S>,
            },
            (Opt::Async, Opt::None) => quote! {
                Self: ::yage_core::component::sync::AsyncComponent<State = S>,
            },
            (Opt::Sync, Opt::Dynamic) => quote! {
                Self: ::yage_core::component::DynamicComponent<State = S>,
            },
            (Opt::Async, Opt::Dynamic) => quote! {
                Self: ::yage_core::component::sync::AsyncDynamicComponent<State = S>,
            },
            (Opt::Stateless, Opt::None) => quote! {
                Self: ::yage_core::component::stateless::Stateless,
            },
            (Opt::Stateless, Opt::Dynamic) => quote! {
                Self: ::yage_core::component::stateless::StatelessDyn,
            },
            _ => quote! {
                compile_error!("inconsistent attribute state")
            },
        };

        syn::parse2(tokens)
    }

    fn function_for_options(tokens: [Self; 2]) -> Result<TokenStream, syn::Error> {
        todo!()
    }
}

pub(crate) fn impl_component_convert(
    attrs: Vec<Attribute>,
    name: Ident,
    generics: &mut Generics,
) -> syn::Result<TokenStream> {
    let mut array: Array<Opt, 2> = Array::new();

    for atttribute in attrs {
        let count = Cell::new(0);

        let mut update_attr = |opt| -> syn::Result<()> {
            array
                .push(opt)
                .map_err(|_| syn::Error::new(Span::call_site(), "too many papi UwU"))?;
            count.set(count.get() + 1);
            Ok(())
        };

        if atttribute.path().is_ident("component") {
            atttribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("async") {
                    return update_attr(Opt::Async);
                }

                if meta.path.is_ident("sync") {
                    return update_attr(Opt::Sync);
                }

                if meta.path.is_ident("stateless") {
                    return update_attr(Opt::Stateless);
                }

                if meta.path.is_ident("dynamic") && count.get() == 1 {
                    return update_attr(Opt::Dynamic);
                }

                if meta.path.is_ident("dynamic") && count.get() != 1 {
                    return Err(syn::Error::new(Span::call_site(), "dumbass!"));
                }

                Ok(())
            })?;
            array.push_rest_with(Opt::None);
        }
    }

    let arr = unsafe { array.assume_init_unchecked() };

    generics
        .params
        .push(syn::GenericParam::Type(syn::TypeParam {
            attrs: vec![],
            ident: Ident::new("S", name.span()),
            bounds: syn::punctuated::Punctuated::new(),
            colon_token: None,
            eq_token: None,
            default: None,
        }));

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let where_clause = where_clause.cloned().get_or_insert(syn::WhereClause {
        where_token: Where { span: name.span() },
        predicates: syn::punctuated::Punctuated::new(),
    });

    let self_pred = Opt::bound_for_options(arr)?;

    where_clause.predicates.push(self_pred);

    let tokens = quote! {
        impl #impl_generics ::yage_core::component::__private::_ComponentConversion<S> for #name #ty_generics
        #where_clause
        {


        }
    };
    todo!()
}
