use darling::FromMeta;
use darling::ast::NestedMeta;
use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::{ToTokens, quote};
use syn::spanned::Spanned;

#[derive(FromMeta)]
struct DeviceArgs {
    heap_size: Option<syn::Expr>,
    address: Option<syn::Expr>,
    mcu_config: Option<syn::Expr>,
    internal_transport_config: Option<syn::Expr>,
    external_transport_config: Option<syn::Expr>,
}

pub fn device(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = NestedMeta::parse_meta_list(attr.into()).unwrap_or_else(|e| abort!("{}", e));
    let args = DeviceArgs::from_list(&attr_args).unwrap_or_else(|e| abort!("{}", e));

    let function: syn::ItemFn = syn::parse(item).unwrap_or_else(|e| abort!("{}", e.to_string()));
    let function_ident = &function.sig.ident;

    let invalid_device_type_error = "Parameter must be of type `Context`";
    let invalid_device_argument_error = "Expected device type as argument";

    let (device_type_path, transports_type_path, state_type_path) =
        match function.sig.inputs.first() {
            Some(syn::FnArg::Typed(pattern)) => match &*pattern.ty {
                syn::Type::Path(v) => {
                    let last_segment = &v.path.segments.last().unwrap();
                    match &last_segment.arguments {
                        syn::PathArguments::AngleBracketed(v) => {
                            if v.args.len() != 3 {
                                abort!(v.args.span(), "Expected 3 type arguments");
                            }
                            let mut iter = v.args.iter();
                            let a = match iter.next().unwrap() {
                                syn::GenericArgument::Type(syn::Type::Path(path)) => path,
                                _ => abort!(v.span(), invalid_device_argument_error),
                            };
                            let b = match iter.next().unwrap() {
                                syn::GenericArgument::Type(syn::Type::Path(path)) => path,
                                _ => abort!(v.span(), invalid_device_argument_error),
                            };
                            let c = match iter.next().unwrap() {
                                syn::GenericArgument::Type(syn::Type::Path(path)) => path,
                                _ => abort!(v.span(), invalid_device_argument_error),
                            };
                            (a, b, c)
                        }
                        _ => abort!(v.span(), invalid_device_argument_error),
                    }
                }
                _ => abort!(pattern.ty.span(), invalid_device_type_error),
            },
            Some(arg @ syn::FnArg::Receiver(_)) => abort!(arg.span(), invalid_device_type_error),
            None => abort!(function.sig.inputs.span(), invalid_device_type_error),
        };

    let heap_size = match args.heap_size {
        Some(v) => v.to_token_stream(),
        None => quote! {
            <<#device_type_path as ::lokey::Device>::Mcu as ::lokey::mcu::HeapSize>::DEFAULT_HEAP_SIZE
        },
    };

    let address = match args.address {
        Some(v) => v.to_token_stream(),
        None => quote! { <#device_type_path as ::lokey::Device>::DEFAULT_ADDRESS },
    };

    let modify_mcu_config = match args.mcu_config {
        Some(v) => quote! { #v(__config); },
        None => quote! {},
    };
    let modify_internal_transport_config = match args.internal_transport_config {
        Some(v) => quote! { #v(__config); },
        None => quote! {},
    };
    let modify_external_transport_config = match args.external_transport_config {
        Some(v) => quote! { #v(__config); },
        None => quote! {},
    };

    quote! {
        extern crate alloc;

        #[global_allocator]
        static HEAP: ::lokey::embedded_alloc::LlffHeap = ::lokey::embedded_alloc::LlffHeap::empty();

        #[::lokey::embassy_executor::main]
        async fn main(spawner: ::lokey::embassy_executor::Spawner) {
            fn __modify_mcu_config(
                __config: &mut <<#device_type_path as ::lokey::Device>::Mcu as ::lokey::mcu::McuInit>::Config
            ) {
                #modify_mcu_config
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

            // Initialize allocator
            {
                use ::core::mem::MaybeUninit;
                const HEAP_SIZE: usize = #heap_size;
                static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
                unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
            }

            let address: ::lokey::Address = #address;

            // Get MCU config
            let mut mcu_config = <#device_type_path as ::lokey::Device>::mcu_config();
            __modify_mcu_config(&mut mcu_config);

            // Get internal transport config
            let mut internal_transport_config = <#transports_type_path as ::lokey::Transports<<#device_type_path as ::lokey::Device>::Mcu>>::internal_transport_config();
            __modify_internal_transport_config(&mut internal_transport_config);

            // Get external transport config
            let mut external_transport_config = <#transports_type_path as ::lokey::Transports<<#device_type_path as ::lokey::Device>::Mcu>>::external_transport_config();
            __modify_external_transport_config(&mut external_transport_config);

            // Create MCU
            let mcu = <<#device_type_path as ::lokey::Device>::Mcu as ::lokey::mcu::McuInit>::create(
                mcu_config,
                address,
                spawner
            );
            let mcu = ::alloc::boxed::Box::leak(::alloc::boxed::Box::new(mcu));

            // Create channels
            let internal_channel = {
                let transport = <::lokey::internal::DeviceTransport::<#device_type_path, #transports_type_path> as ::lokey::internal::Transport>::create(
                    internal_transport_config,
                    mcu,
                    address,
                    spawner,
                )
                .await;
                let transport = ::alloc::boxed::Box::leak(::alloc::boxed::Box::new(transport));
                ::lokey::internal::Channel::new(transport)
            };

            let external_channel = {
                let transport = <::lokey::external::DeviceTransport::<#device_type_path, #transports_type_path> as ::lokey::external::Transport>::create(
                    external_transport_config,
                    mcu,
                    address,
                    spawner,
                    internal_channel.as_dyn()
                )
                .await;
                let transport = ::alloc::boxed::Box::leak(::alloc::boxed::Box::new(transport));
                ::lokey::external::Channel::new(transport)
            };

            let state = ::alloc::boxed::Box::leak(::alloc::boxed::Box::new(
                <#state_type_path as ::core::default::Default>::default()
            ));

            let context = ::lokey::Context {
                spawner,
                address,
                mcu,
                external_channel,
                internal_channel,
                state,
            };

            ::lokey::mcu::McuInit::run(mcu, context.as_dyn());

            #[::lokey::embassy_executor::task]
            async fn __run_internal_channel(channel: ::lokey::internal::Channel<::lokey::internal::DeviceTransport<#device_type_path, #transports_type_path>>) {
                channel.run().await;
            }
            spawner.must_spawn(__run_internal_channel(internal_channel));

            #[::lokey::embassy_executor::task]
            async fn __run_external_channel(channel: ::lokey::external::Channel<::lokey::external::DeviceTransport<#device_type_path, #transports_type_path>>) {
                channel.run().await;
            }
            spawner.must_spawn(__run_external_channel(external_channel));

            #function

            #function_ident(context).await
        }
    }
    .into()
}
