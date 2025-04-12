use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;
use syn::{DeriveInput, parse_macro_input};

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

    quote! {
        #(
            impl ::lokey::State<#field_types> for #ident {
                fn get(&self) -> &#field_types {
                    &self.#field_accessors
                }

                fn get_mut(&mut self) -> &mut #field_types {
                    &mut self.#field_accessors
                }
            }
        )*

        impl ::lokey::StateContainer for #ident {
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
        }
    }
    .into()
}
