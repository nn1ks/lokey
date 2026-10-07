use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;
use syn::{DeriveInput, parse_macro_input};

pub fn tx_message_derive(item: TokenStream) -> TokenStream {
    let DeriveInput {
        vis, ident, data, ..
    } = parse_macro_input!(item);
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
        syn::Ident::new(&format!("{}UsbTxMessageService", ident), Span::call_site());

    let rx_message_service_container_ident = syn::Ident::new(
        &format!("{}UsbRxMessageServiceContainer", ident),
        Span::call_site(),
    );

    quote! {
        impl ::lokey_usb::external::TxMessage for #ident {
            type MessageService<'d, D: ::lokey_usb::embassy_usb::driver::Driver<'d> + 'd> = #message_service_ident<'d, D>;
        }

        #vis struct #message_service_ident<'d, D: ::lokey_usb::embassy_usb::driver::Driver<'d> + 'd> {
            services: (#(<#variant_types as ::lokey_usb::external::TxMessage>::MessageService<'d, D>),*),
        }

        #vis struct #rx_message_service_container_ident<'d, D: ::lokey_usb::embassy_usb::driver::Driver<'d> + 'd> {
            services: (#(<<#variant_types as ::lokey_usb::external::TxMessage>::MessageService<'d, D> as ::lokey_usb::external::InitTxMessageService<'d, D>>::RxMessageServiceContainer),*),
        }

        impl<'d, D: ::lokey_usb::embassy_usb::driver::Driver<'d> + 'd> ::lokey_usb::external::RxMessageServiceContainer<'d> for #rx_message_service_container_ident<'d, D> {
            unsafe fn rx_message_service<T: 'd>(&self) -> Option<&T> {
                #(
                    if let Some(value) = self.services.#field_indices.rx_message_service::<T>() {
                        return Some(value);
                    }
                )*
                return None;
            }

            unsafe fn take_rx_message_service<T: 'd>(&mut self) -> Option<T> {
                #(
                    if let Some(value) = self.services.#field_indices.take_rx_message_service::<T>() {
                        return Some(value);
                    }
                )*
                return None;
            }
        }

        impl<'d, D: ::lokey_usb::embassy_usb::driver::Driver<'d> + 'd> ::lokey_usb::external::InitTxMessageService<'d, D> for #message_service_ident<'d, D> {
            type Params = (
                #(<<#variant_types as ::lokey_usb::external::TxMessage>::MessageService<'d, D> as ::lokey_usb::external::InitTxMessageService<'d, D>>::Params),*
            );
            type RxMessageServiceContainer = #rx_message_service_container_ident<'d, D>;

            fn create_params() -> Self::Params {
                (#(
                    <<#variant_types as ::lokey_usb::external::TxMessage>::MessageService<'d, D> as ::lokey_usb::external::InitTxMessageService<'d, D>>::create_params()
                ),*)
            }

            fn init(builder: &mut ::lokey_usb::embassy_usb::Builder<'d, D>, params: &'d mut Self::Params) -> (Self, Self::RxMessageServiceContainer) {
                let value = (#(
                    <<#variant_types as ::lokey_usb::external::TxMessage>::MessageService<'d, D> as ::lokey_usb::external::InitTxMessageService<'d, D>>::init(builder, &mut params.#field_indices)
                ),*);
                (
                    Self { services: (#(value.#field_indices.0),*) },
                    #rx_message_service_container_ident { services: (#(value.#field_indices.1),*) }
                )
            }
        }

        impl<'d, D: ::lokey_usb::embassy_usb::driver::Driver<'d> + 'd> ::lokey_usb::external::TxMessageService<#ident> for #message_service_ident<'d, D> {
            async fn send(&mut self, message: #ident) {
                match message {
                    #(#ident::#variant_names(v) => self.services.#field_indices.send(v).await),*
                }
            }
        }
    }
    .into()
}
