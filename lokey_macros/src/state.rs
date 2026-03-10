use darling::FromField;
use darling::util::Flag;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;
use syn::{DeriveInput, parse_macro_input};

#[derive(Debug, FromField)]
#[darling(attributes(state))]
struct FieldAttrs {
    query: Flag,
}

pub fn state_derive(item: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(item);
    let data_struct = match data {
        syn::Data::Struct(v) => v,
        syn::Data::Enum(_) => {
            return syn::Error::new(
                Span::call_site(),
                "State derive macro can not be used on enums",
            )
            .into_compile_error()
            .into();
        }
        syn::Data::Union(_) => {
            return syn::Error::new(
                Span::call_site(),
                "State derive macro can not be used on unions",
            )
            .into_compile_error()
            .into();
        }
    };

    let mut errors = darling::Error::accumulator();
    let field_opts: Vec<_> = match &data_struct.fields {
        syn::Fields::Named(v) => v
            .named
            .iter()
            .filter_map(|v| errors.handle(FieldAttrs::from_field(v)))
            .collect(),
        syn::Fields::Unnamed(v) => v
            .unnamed
            .iter()
            .filter_map(|v| errors.handle(FieldAttrs::from_field(v)))
            .collect(),
        syn::Fields::Unit => Vec::new(),
    };
    if let Err(e) = errors.finish() {
        return e.write_errors().into();
    }

    let field_types: Vec<_> = match &data_struct.fields {
        syn::Fields::Named(v) => v.named.iter().map(|v| &v.ty).collect(),
        syn::Fields::Unnamed(v) => v.unnamed.iter().map(|v| &v.ty).collect(),
        syn::Fields::Unit => Vec::new(),
    };
    let field_accessors: Vec<_> = match &data_struct.fields {
        syn::Fields::Named(v) => v
            .named
            .iter()
            .map(|field| field.ident.clone().unwrap())
            .collect(),
        syn::Fields::Unnamed(v) => v
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, field)| syn::Ident::new(&i.to_string(), field.span()))
            .collect(),
        syn::Fields::Unit => Vec::new(),
    };

    let field_types_and_accesors_with_state_query = field_opts
        .iter()
        .zip(&field_types)
        .zip(&field_accessors)
        .filter(|((opts, _), _)| opts.query.is_present())
        .map(|((_, ty), accessor)| (ty, accessor))
        .collect::<Vec<_>>();

    let query_state_impls = field_types_and_accesors_with_state_query
        .iter()
        .map(|(ty, accessor)| {
            quote! {
                impl<'a> ::lokey::state::QueryState<'a, <#ty as ::lokey::state::ToStateQuery>::Query<'a>> for #ident {
                    fn query(&'a self) -> <#ty as ::lokey::state::ToStateQuery>::Query<'a> {
                        ::lokey::state::ToStateQuery::to_query(&self.#accessor)
                    }
                }
            }
        })
        .collect::<proc_macro2::TokenStream>();

    let try_query_state_branches = field_types_and_accesors_with_state_query
        .iter()
        .map(|(ty, accessor)| {
            quote! {
                if ::lokey::typeid::of::<T>() == ::lokey::typeid::of::<<#ty as ::lokey::state::ToStateQuery>::Query<'static>>() {
                    let query: <#ty as ::lokey::state::ToStateQuery>::Query<'_> = ::lokey::state::ToStateQuery::to_query(&self.#accessor);
                    let new_query = unsafe { ::core::mem::transmute_copy(&query) };
                    #[allow(clippy::forget_non_drop)]
                    ::core::mem::forget(query);
                    return ::core::option::Option::Some(::lokey::state::StateQueryRef::new(new_query));
                }
            }
        })
        .collect::<proc_macro2::TokenStream>();

    quote! {
        #(
            impl ::lokey::state::GetState<#field_types> for #ident {
                fn get(&self) -> &#field_types {
                    &self.#field_accessors
                }

                fn get_mut(&mut self) -> &mut #field_types {
                    &mut self.#field_accessors
                }
            }
        )*

        #query_state_impls

        impl ::lokey::state::StateContainer for #ident {
            fn try_get_raw(&self, type_id: ::core::any::TypeId) -> ::core::option::Option<&dyn ::core::any::Any> {
                #(
                    if type_id == ::core::any::TypeId::of::<#field_types>() {
                        return ::core::option::Option::Some(&self.#field_accessors);
                    }
                )*
                ::core::option::Option::None
            }

            fn try_get_mut_raw(&mut self, type_id: ::core::any::TypeId) -> ::core::option::Option<&mut dyn ::core::any::Any> {
                #(
                    if type_id == ::core::any::TypeId::of::<#field_types>() {
                        return ::core::option::Option::Some(&mut self.#field_accessors);
                    }
                )*
                ::core::option::Option::None
            }

            fn try_query<T>(&self) -> ::core::option::Option<::lokey::state::StateQueryRef<'_, T>> {
                #try_query_state_branches
                ::core::option::Option::None
            }
        }
    }
    .into()
}
