use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;
use syn::{DeriveInput, parse_macro_input};

pub fn tx_message_derive(item: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(item);
    let data_enum = match data {
        syn::Data::Enum(v) => v,
        syn::Data::Struct(_) => {
            return syn::Error::new(
                Span::call_site(),
                "Message derive macro can not be used on structs",
            )
            .into_compile_error()
            .into();
        }
        syn::Data::Union(_) => {
            return syn::Error::new(
                Span::call_site(),
                "Message derive macro can not be used on unions",
            )
            .into_compile_error()
            .into();
        }
    };

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
                        "TxMessage derive macro can only be used on enum variants with exactly one unnamed field",
                    ))
                }
                else {
                    Ok(fields_unnamed.unnamed.first().unwrap())
                }
            },
            syn::Fields::Named(_) => Err(syn::Error::new(
                v.span(),
                "TxMessage derive macro can not be used on enum variants with named fields",
            )),
            syn::Fields::Unit => Err(syn::Error::new(
                v.span(),
                "TxMessage derive macro can not be used on unit enum variants",
            )),
        })
        .collect::<Result<Vec<_>, _>>();
    let variant_fields = match variant_fields {
        Ok(v) => v,
        Err(e) => return e.into_compile_error().into(),
    };

    let variant_types = variant_fields.iter().map(|f| &f.ty).collect::<Vec<_>>();

    let field_indices = (0..variant_types.len())
        .map(syn::Index::from)
        .collect::<Vec<_>>();

    let message_service_ident =
        syn::Ident::new(&format!("{}BleTxMessageService", ident), Span::call_site());

    let len_service_uuids_16 = variant_types
        .iter()
        .fold(quote! { ::lokey_ble::typenum::U0 }, |acc, variant_type| {
            quote! { ::lokey_ble::typenum::Sum<<#variant_type as ::lokey_ble::external::TxMessage>::LenServiceUuids16, #acc> }
        });

    let len_service_uuids_128 = variant_types
        .iter()
        .fold(quote! { ::lokey_ble::typenum::U0 }, |acc, variant_type| {
            quote! { ::lokey_ble::typenum::Sum<<#variant_type as ::lokey_ble::external::TxMessage>::LenServiceUuids128, #acc> }
        });

    quote! {
        impl ::lokey_ble::external::TxMessage for #ident {
            type MessageService = #message_service_ident;

            const ATTRIBUTE_COUNT: usize = 0 #(+ <#variant_types as ::lokey_ble::external::TxMessage>::ATTRIBUTE_COUNT)*;
            const CCCD_COUNT: usize = 0 #(+ <#variant_types as ::lokey_ble::external::TxMessage>::CCCD_COUNT)*;

            type LenServiceUuids16 = #len_service_uuids_16;
            type LenServiceUuids128 = #len_service_uuids_128;

            fn service_uuids_16() -> ::lokey_ble::generic_array::GenericArray<[u8; 2], Self::LenServiceUuids16> {
                ::lokey_ble::generic_array::sequence::Concat::concat(
                    #( <#variant_types as ::lokey_ble::external::TxMessage>::service_uuids_16() ),*
                )
            }

            fn service_uuids_128() -> ::lokey_ble::generic_array::GenericArray<[u8; 16], Self::LenServiceUuids128> {
                ::lokey_ble::generic_array::sequence::Concat::concat(
                    #( <#variant_types as ::lokey_ble::external::TxMessage>::service_uuids_128() ),*
                )
            }
        }

        struct #message_service_ident {
            services: (#(<#variant_types as ::lokey_ble::external::TxMessage>::MessageService),*),
        }

        impl ::lokey_ble::external::InitMessageService for #message_service_ident {
            fn init<const ATT_MAX: usize>(
                attribute_table: &mut ::lokey_ble::trouble_host::prelude::AttributeTable<'static, ::lokey_ble::embassy_sync::blocking_mutex::raw::NoopRawMutex, ATT_MAX>,
            ) -> Self {
                Self {
                    services: (
                        #(<<#variant_types as ::lokey_ble::external::TxMessage>::MessageService as ::lokey_ble::external::InitMessageService>::init(attribute_table)),*
                    ),
                }
            }
        }

        impl ::lokey_ble::external::TxMessageService<#ident> for #message_service_ident {
            async fn send<'stack, 'server>(
                &self,
                message: #ident,
                connection: &::lokey_ble::trouble_host::gatt::GattConnection<'stack, 'server, ::lokey_ble::trouble_host::prelude::DefaultPacketPool>,
            ) {
                match message {
                    #(#ident::#variant_names(v) => self.services.#field_indices.send(v, connection).await),*
                }
            }
        }
    }
    .into()
}
