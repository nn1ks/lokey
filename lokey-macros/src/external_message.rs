use darling::FromDeriveInput;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
use syn::{DeriveInput, parse_macro_input};

#[derive(FromDeriveInput)]
#[darling(attributes(external_message))]
struct ExternalMessageArgs {
    #[darling(default, rename = "crate")]
    crate_: Option<syn::Path>,
}

pub fn external_message_derive(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);
    let args = match ExternalMessageArgs::from_derive_input(&input) {
        Ok(args) => args,
        Err(e) => return e.write_errors().into(),
    };

    let crate_path: TokenStream2 = args
        .crate_
        .as_ref()
        .map(|p| quote! { #p })
        .unwrap_or_else(|| quote! { ::lokey });

    match input.data {
        syn::Data::Enum(data_enum) => {
            external_message_derive_enum(input.ident, data_enum, crate_path)
        }
        syn::Data::Struct(_) => external_message_derive_struct(input.ident, crate_path),
        syn::Data::Union(_) => external_message_derive_struct(input.ident, crate_path),
    }
}

fn external_message_derive_struct(ident: syn::Ident, crate_path: TokenStream2) -> TokenStream {
    quote! {
        impl #crate_path::external::Message for #ident {
            fn has_inner_message<M: #crate_path::external::Message>() -> bool {
                false
            }

            fn inner_message<M: #crate_path::external::Message>(&self) -> ::core::option::Option<&M> {
                ::core::option::Option::None
            }

            fn try_from_inner_message(value: &dyn ::core::any::Any) -> ::core::result::Result<Self, #crate_path::external::MismatchedMessageType>
            where
                Self: ::core::marker::Sized,
            {
                ::core::result::Result::Err(#crate_path::external::MismatchedMessageType)
            }
        }
    }
    .into()
}

fn external_message_derive_enum(
    ident: syn::Ident,
    data_enum: syn::DataEnum,
    crate_path: TokenStream2,
) -> TokenStream {
    let variant_names = data_enum
        .variants
        .iter()
        .map(|v| &v.ident)
        .collect::<Vec<_>>();

    let variant_fields = data_enum
        .variants
        .iter()
        .map(|v| match &v.fields {
            syn::Fields::Unnamed(fields_unnamed) => {
                if fields_unnamed.unnamed.len() != 1 {
                    Err(syn::Error::new(
                        v.span(),
                        "Message derive macro can only be used on enum variants with exactly one unnamed field",
                    ))
                }
                else {
                    Ok(fields_unnamed.unnamed.first().unwrap())
                }
            },
            syn::Fields::Named(_) => Err(syn::Error::new(
                v.span(),
                "Message derive macro can not be used on enum variants with named fields",
            )),
            syn::Fields::Unit => Err(syn::Error::new(
                v.span(),
                "Message derive macro can not be used on unit enum variants",
            )),
        })
        .collect::<Result<Vec<_>, _>>();
    let variant_fields = match variant_fields {
        Ok(v) => v,
        Err(e) => return e.into_compile_error().into(),
    };

    let variant_types = variant_fields.iter().map(|f| &f.ty).collect::<Vec<_>>();

    quote! {
        impl #crate_path::external::Message for #ident {
            fn has_inner_message<M: #crate_path::external::Message>() -> bool {
                false
                #(
                    || ::core::any::TypeId::of::<M>() == ::core::any::TypeId::of::<#variant_types>()
                    || <#variant_types as #crate_path::external::Message>::has_inner_message::<M>()
                )*
            }

            fn inner_message<M: #crate_path::external::Message>(&self) -> ::core::option::Option<&M> {
                #(
                    if ::core::any::TypeId::of::<M>() == ::core::any::TypeId::of::<#variant_types>() {
                        if let Self::#variant_names(v) = self {
                            return (v as &dyn ::core::any::Any).downcast_ref();
                        }
                    }
                )*
                match self {
                    #(
                        Self::#variant_names(v) => {
                            if let ::core::option::Option::Some(v) = <#variant_types as #crate_path::external::Message>::inner_message::<M>(v) {
                                return ::core::option::Option::Some(v);
                            }
                        }
                    )*
                }
                ::core::option::Option::None
            }

            fn try_from_inner_message(value: &dyn ::core::any::Any) -> ::core::result::Result<Self, #crate_path::external::MismatchedMessageType>
            where
                Self: ::core::marker::Sized,
            {
                #(
                    if let ::core::option::Option::Some(v) = value.downcast_ref::<#variant_types>() {
                        return ::core::result::Result::Ok(Self::#variant_names(::core::clone::Clone::clone(v)));
                    }
                )*
                #(
                    if let ::core::result::Result::Ok(v) = <#variant_types as #crate_path::external::Message>::try_from_inner_message(value) {
                        return ::core::result::Result::Ok(Self::#variant_names(v));
                    }
                )*
                ::core::result::Result::Err(#crate_path::external::MismatchedMessageType)
            }
        }
    }
    .into()
}
