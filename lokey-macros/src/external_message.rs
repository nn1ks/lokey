use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{DeriveInput, parse_macro_input};

pub fn external_message_derive(item: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(item);
    match data {
        syn::Data::Enum(data_enum) => external_message_derive_enum(ident, data_enum),
        syn::Data::Struct(_) => external_message_derive_struct(ident),
        syn::Data::Union(_) => external_message_derive_struct(ident),
    }
}

fn external_message_derive_struct(ident: syn::Ident) -> TokenStream {
    quote! {
        impl ::lokey::external::Message for #ident {
            fn has_inner_message<M: ::lokey::external::Message>() -> bool {
                false
            }

            fn inner_message<M: ::lokey::external::Message>(&self) -> ::core::option::Option<&M> {
                ::core::option::Option::None
            }

            fn try_from_inner_message(value: &dyn ::core::any::Any) -> ::core::result::Result<Self, ::lokey::external::MismatchedMessageType>
            where
                Self: ::core::marker::Sized,
            {
                ::core::result::Result::Err(::lokey::external::MismatchedMessageType)
            }
        }
    }
    .into()
}

fn external_message_derive_enum(ident: syn::Ident, data_enum: syn::DataEnum) -> TokenStream {
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
        impl ::lokey::external::Message for #ident {
            fn has_inner_message<M: ::lokey::external::Message>() -> bool {
                false
                #(
                    || ::core::any::TypeId::of::<M>() == ::core::any::TypeId::of::<#variant_types>()
                    || <#variant_types as ::lokey::external::Message>::has_inner_message::<M>()
                )*
            }

            fn inner_message<M: ::lokey::external::Message>(&self) -> ::core::option::Option<&M> {
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
                            if let ::core::option::Option::Some(v) = <#variant_types as ::lokey::external::Message>::inner_message::<M>(v) {
                                return ::core::option::Option::Some(v);
                            }
                        }
                    )*
                }
                ::core::option::Option::None
            }

            fn try_from_inner_message(value: &dyn ::core::any::Any) -> ::core::result::Result<Self, ::lokey::external::MismatchedMessageType>
            where
                Self: ::core::marker::Sized,
            {
                #(
                    if let ::core::option::Option::Some(v) = value.downcast_ref::<#variant_types>() {
                        return ::core::result::Result::Ok(Self::#variant_names(::core::clone::Clone::clone(v)));
                    }
                )*
                #(
                    if let ::core::result::Result::Ok(v) = <#variant_types as ::lokey::external::Message>::try_from_inner_message(value) {
                        return ::core::result::Result::Ok(Self::#variant_names(v));
                    }
                )*
                ::core::result::Result::Err(::lokey::external::MismatchedMessageType)
            }
        }
    }
    .into()
}
