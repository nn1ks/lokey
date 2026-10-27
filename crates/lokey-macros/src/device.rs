use darling::FromMeta;
use darling::ast::NestedMeta;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::spanned::Spanned;

#[derive(FromMeta)]
struct DeviceArgs {
    address: Option<syn::Expr>,
    mcu_config: Option<syn::Expr>,
    storage_config: Option<syn::Expr>,
    internal_transport_config: Option<syn::Expr>,
    external_transport_config: Option<syn::Expr>,
    message_override: Option<syn::Expr>,
}

const INVALID_DEVICE_TYPE_ERROR: &str = "Parameter must be of type `Context`";
const INVALID_ARGUMENT_TYPE_ERROR: &str = "Unexpected generic argument, expected type argument";

fn parse_context_generic_arguments(
    v: &syn::AngleBracketedGenericArguments,
) -> syn::Result<(syn::TypePath, syn::TypePath, syn::TypePath)> {
    if v.args.len() != 3 {
        return Err(syn::Error::new(v.args.span(), "Expected 3 type arguments"));
    }
    let mut iter = v.args.iter();
    let a = match iter.next().unwrap() {
        syn::GenericArgument::Type(syn::Type::Path(path)) => path.clone(),
        _ => {
            return Err(syn::Error::new(v.span(), INVALID_ARGUMENT_TYPE_ERROR));
        }
    };
    let b = match iter.next().unwrap() {
        syn::GenericArgument::Type(syn::Type::Path(path)) => path.clone(),
        _ => {
            return Err(syn::Error::new(v.span(), INVALID_ARGUMENT_TYPE_ERROR));
        }
    };
    let c = match iter.next().unwrap() {
        syn::GenericArgument::Type(syn::Type::Path(path)) => path.clone(),
        _ => {
            return Err(syn::Error::new(v.span(), INVALID_ARGUMENT_TYPE_ERROR));
        }
    };
    Ok((a, b, c))
}

pub fn device(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream2> {
    let attr_args = NestedMeta::parse_meta_list(attr.into())?;
    let args = DeviceArgs::from_list(&attr_args)?;

    let function: syn::ItemFn = syn::parse(item)?;
    let function_ident = &function.sig.ident;

    let context_param = if function.sig.inputs.len() == 2 {
        &function.sig.inputs[0]
    } else {
        return Err(syn::Error::new(
            function.sig.inputs.span(),
            "Expected 2 parameters",
        ));
    };

    let (device_type_path, transports_type_path, state_type_path) = match &context_param {
        syn::FnArg::Typed(pattern) => match &*pattern.ty {
            syn::Type::Path(v) => {
                let last_segment = &v.path.segments.last().unwrap();
                match &last_segment.arguments {
                    syn::PathArguments::AngleBracketed(v) => parse_context_generic_arguments(v)?,
                    _ => {
                        return Err(syn::Error::new(v.span(), INVALID_ARGUMENT_TYPE_ERROR));
                    }
                }
            }
            _ => {
                return Err(syn::Error::new(
                    pattern.ty.span(),
                    INVALID_DEVICE_TYPE_ERROR,
                ));
            }
        },
        syn::FnArg::Receiver(_) => {
            return Err(syn::Error::new(
                context_param.span(),
                INVALID_DEVICE_TYPE_ERROR,
            ));
        }
    };

    let address = match args.address {
        Some(v) => v.to_token_stream(),
        None => quote! { <#device_type_path as ::lokey::Device>::DEFAULT_ADDRESS },
    };

    let modify_mcu_config = match args.mcu_config {
        Some(v) => quote! { (#v)(__config); },
        None => quote! {},
    };
    let modify_storage_config = match args.storage_config {
        Some(v) => quote! { (#v)(__config); },
        None => quote! {},
    };
    let modify_internal_transport_config = match args.internal_transport_config {
        Some(v) => quote! { (#v)(__config); },
        None => quote! {},
    };
    let modify_external_transport_config = match args.external_transport_config {
        Some(v) => quote! { (#v)(__config); },
        None => quote! {},
    };

    let message_override = match args.message_override {
        Some(v) => quote! { #v },
        None => {
            let tx_message_type = quote! {
                <::lokey::external::DeviceTransport<#device_type_path, #transports_type_path> as ::lokey::external::Transport>::TxMessage
            };
            quote! { ::lokey::external::IdentityOverride::<#tx_message_type>::new() }
        }
    };

    Ok(quote! {
        #[::embassy_executor::main]
        async fn main(spawner: ::embassy_executor::Spawner) {
            fn __modify_mcu_config(
                __config: &mut <<#device_type_path as ::lokey::Device>::Mcu as ::lokey::Mcu>::Config
            ) {
                #modify_mcu_config
            }

            fn __modify_storage_config(
                __config: &mut <#device_type_path as ::lokey::Device>::StorageConfig
            ) {
                #modify_storage_config
            }

            fn __modify_internal_transport_config(
                __config: &mut <::lokey::internal::DeviceTransport::<#device_type_path, #transports_type_path> as ::lokey::internal::Transport>::Config
            ) {
                #modify_internal_transport_config
            }

            fn __modify_external_transport_config(
                __config: &mut <::lokey::external::DeviceTransport::<#device_type_path, #transports_type_path> as ::lokey::external::Transport>::Config
            ) {
                #modify_external_transport_config
            }

            let address: ::lokey::Address = #address;

            // Get MCU config
            let mut mcu_config = <#device_type_path as ::lokey::Device>::mcu_config();
            __modify_mcu_config(&mut mcu_config);

            // Get storage config
            let mut storage_config = <#device_type_path as ::lokey::Device>::storage_config();
            __modify_storage_config(&mut storage_config);

            // Get internal transport config
            let mut internal_transport_config = <#transports_type_path as ::lokey::Transports<<#device_type_path as ::lokey::Device>::Mcu>>::internal_transport_config();
            __modify_internal_transport_config(&mut internal_transport_config);

            // Get external transport config
            let mut external_transport_config = <#transports_type_path as ::lokey::Transports<<#device_type_path as ::lokey::Device>::Mcu>>::external_transport_config();
            __modify_external_transport_config(&mut external_transport_config);

            // Create MCU and storage
            let (mcu, storage) = {
                static MCU: ::lokey::static_cell::StaticCell<<#device_type_path as ::lokey::Device>::Mcu> = ::lokey::static_cell::StaticCell::new();
                static STORAGE: ::lokey::static_cell::StaticCell<<<#device_type_path as ::lokey::Device>::Mcu as ::lokey::McuCreator<<#device_type_path as ::lokey::Device>::StorageConfig>>::Storage> = ::lokey::static_cell::StaticCell::new();
                let (mcu, storage) = <<#device_type_path as ::lokey::Device>::Mcu as ::lokey::McuCreator<<#device_type_path as ::lokey::Device>::StorageConfig>>::create(spawner, mcu_config, address, storage_config).await;
                (MCU.init(mcu), STORAGE.init(storage))
            };

            // Create channels
            let internal_channel = {
                static CHANNEL: ::lokey::static_cell::StaticCell<
                    ::lokey::internal::Channel<
                        ::lokey::internal::DeviceTransport::<#device_type_path, #transports_type_path>
                    >
                > = ::lokey::static_cell::StaticCell::new();
                CHANNEL.init(
                    ::lokey::internal::Channel::new(
                        <::lokey::internal::DeviceTransport::<#device_type_path, #transports_type_path> as ::lokey::internal::Transport>::create(
                            internal_transport_config,
                            mcu,
                            address,
                        )
                        .await
                    )
                )
            };

            let external_channel = {
                static CHANNEL: ::lokey::static_cell::StaticCell<
                    ::lokey::external::Channel<
                        ::lokey::external::DeviceTransport::<#device_type_path, #transports_type_path>
                    >
                > = ::lokey::static_cell::StaticCell::new();
                CHANNEL.init(
                    ::lokey::external::Channel::new(
                        <::lokey::external::DeviceTransport::<#device_type_path, #transports_type_path> as ::lokey::external::Transport>::create(
                            external_transport_config,
                            mcu,
                            address,
                            internal_channel,
                        )
                        .await
                    )
                )
            };

            let state = {
                static STATE: ::lokey::static_cell::StaticCell<#state_type_path> = ::lokey::static_cell::StaticCell::new();
                STATE.init(<#state_type_path as ::core::default::Default>::default())
            };

            let context = ::lokey::Context {
                address,
                mcu,
                storage,
                external_channel,
                internal_channel,
                state,
            };

            #[::embassy_executor::task]
            async fn __run_mcu(mcu: &'static <#device_type_path as ::lokey::Device>::Mcu, context: ::lokey::Context<#device_type_path, #transports_type_path, #state_type_path>) {
                ::lokey::Mcu::run(mcu, context).await;
            }
            spawner.spawn(__run_mcu(mcu, context).unwrap());

            #[::embassy_executor::task]
            async fn __run_internal_channel(
                channel: &'static ::lokey::internal::Channel<::lokey::internal::DeviceTransport<#device_type_path, #transports_type_path>>,
                storage: &'static <<#device_type_path as ::lokey::Device>::Mcu as ::lokey::McuCreator<<#device_type_path as ::lokey::Device>::StorageConfig>>::Storage,
            ) {
                channel.run(storage).await;
            }
            spawner.spawn(__run_internal_channel(internal_channel, storage).unwrap());

            #[::embassy_executor::task]
            async fn __run_external_channel(
                channel: &'static ::lokey::external::Channel<::lokey::external::DeviceTransport<#device_type_path, #transports_type_path>>,
                storage: &'static <<#device_type_path as ::lokey::Device>::Mcu as ::lokey::McuCreator<<#device_type_path as ::lokey::Device>::StorageConfig>>::Storage,
            ) {
                let message_override = #message_override;
                channel.run(storage, message_override).await;
            }
            spawner.spawn(__run_external_channel(external_channel, storage).unwrap());

            #function

            #function_ident(context, spawner).await
        }
    })
}
