use darling::{FromMeta, ast::NestedMeta};
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{ToTokens, quote};
use syn::spanned::Spanned;

#[derive(FromMeta)]
struct DeviceArgs {
    heap_size: Option<syn::Expr>,
    mcu_config: Option<syn::Expr>,
    internal_transport_config: Option<syn::Expr>,
    external_transport_config: Option<syn::Expr>,
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn device(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = NestedMeta::parse_meta_list(attr.into()).unwrap_or_else(|e| abort!("{}", e));
    let args = DeviceArgs::from_list(&attr_args).unwrap_or_else(|e| abort!("{}", e));

    let function: syn::ItemFn = syn::parse(item).unwrap_or_else(|e| abort!("{}", e.to_string()));
    let function_ident = &function.sig.ident;

    let invalid_device_type_error = "Parameter must be of type `Context`";
    let invalid_device_argument_error = "Expected device type as argument";

    let (device_type_path, transports_type_path) = match function.sig.inputs.first() {
        Some(syn::FnArg::Typed(pattern)) => match &*pattern.ty {
            syn::Type::Path(v) => {
                let last_segment = &v.path.segments.last().unwrap();
                match &last_segment.arguments {
                    syn::PathArguments::AngleBracketed(v) => {
                        if v.args.len() != 2 {
                            abort!(v.args.span(), "Expected two type arguments");
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
                        (a, b)
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
                __config: &mut <#transports_type_path as ::lokey::Transports<<#device_type_path as ::lokey::Device>::Mcu>>::InternalTransportConfig
            ) {
                #modify_internal_transport_config
            }

            fn __modify_external_transport_config(
                __config: &mut <#transports_type_path as ::lokey::Transports<<#device_type_path as ::lokey::Device>::Mcu>>::ExternalTransportConfig
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

            // Create MCU
            let mut mcu_config = <#device_type_path as ::lokey::Device>::mcu_config();
            __modify_mcu_config(&mut mcu_config);
            let mcu = <<#device_type_path as ::lokey::Device>::Mcu as ::lokey::mcu::McuInit>::create(
                mcu_config,
                spawner
            );
            let mcu = ::alloc::boxed::Box::leak(::alloc::boxed::Box::new(mcu));

            // Create channels
            let internal_channel = {
                let mut config = <#transports_type_path as ::lokey::Transports<<#device_type_path as ::lokey::Device>::Mcu>>::internal_transport_config();
                __modify_internal_transport_config(&mut config);
                let transport = ::lokey::internal::TransportConfig::init(
                    config,
                    mcu,
                    spawner
                ).await;
                let transport = ::alloc::boxed::Box::leak(::alloc::boxed::Box::new(transport));
                ::lokey::internal::Channel::new(transport, spawner)
            };

            let external_channel = {
                let mut config = <#transports_type_path as ::lokey::Transports<<#device_type_path as ::lokey::Device>::Mcu>>::external_transport_config();
                __modify_external_transport_config(&mut config);
                let transport = ::lokey::external::TransportConfig::init(
                    config,
                    mcu,
                    spawner,
                    internal_channel.as_dyn()
                ).await;
                let transport = ::alloc::boxed::Box::leak(::alloc::boxed::Box::new(transport));
                ::lokey::external::Channel::new(transport)
            };

            let context = ::lokey::Context {
                spawner,
                mcu,
                external_channel,
                internal_channel,
                layer_manager: ::lokey::LayerManager::new(),
            };

            ::lokey::mcu::McuInit::run(mcu, context.as_dyn());

            #function

            #function_ident(context).await
        }
    }
    .into()
}
