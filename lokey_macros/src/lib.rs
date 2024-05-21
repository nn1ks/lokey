use darling::{ast::NestedMeta, FromMeta};
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{parse::Parser, spanned::Spanned};

#[derive(FromMeta)]
struct DeviceArgs {
    heap_size: Option<syn::Expr>,
    mcu_config: Option<syn::Expr>,
    internal_channel_config: Option<syn::Expr>,
    external_channel_config: Option<syn::Expr>,
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

    let device_type_path = match function.sig.inputs.first() {
        Some(syn::FnArg::Typed(pattern)) => match &*pattern.ty {
            syn::Type::Path(ref v) => {
                let last_segment = &v.path.segments.last().unwrap();
                match &last_segment.arguments {
                    syn::PathArguments::AngleBracketed(v) => match v.args.last() {
                        Some(syn::GenericArgument::Type(syn::Type::Path(path))) => {
                            if v.args.len() == 1 {
                                path
                            } else {
                                abort!(v.args.span(), "Expected only one type argument");
                            }
                        }
                        Some(_) | None => abort!(v.span(), invalid_device_argument_error),
                    },
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
    let modify_internal_channel_config = match args.internal_channel_config {
        Some(v) => quote! { #v(__config); },
        None => quote! {},
    };
    let modify_external_channel_config = match args.external_channel_config {
        Some(v) => quote! { #v(__config); },
        None => quote! {},
    };

    quote! {
        extern crate alloc;

        #[global_allocator]
        static HEAP: ::lokey::embedded_alloc::Heap = ::lokey::embedded_alloc::Heap::empty();

        #[::lokey::embassy_executor::main]
        async fn main(spawner: ::lokey::embassy_executor::Spawner) {
            fn modify_mcu_config(
                __config: &mut <<#device_type_path as ::lokey::Device>::Mcu as ::lokey::mcu::McuInit>::Config
            ) {
                #modify_mcu_config
            }

            fn modify_internal_channel_config(
                __config: &mut <#device_type_path as ::lokey::Device>::InternalChannelConfig
            ) {
                #modify_internal_channel_config
            }

            fn modify_external_channel_config(
                __config: &mut <#device_type_path as ::lokey::Device>::ExternalChannelConfig
            ) {
                #modify_external_channel_config
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
            modify_mcu_config(&mut mcu_config);
            let mcu = <<#device_type_path as ::lokey::Device>::Mcu as ::lokey::mcu::McuInit>::create(
                mcu_config,
                spawner
            );
            let mcu = ::alloc::boxed::Box::leak(::alloc::boxed::Box::new(mcu));

            // Create channels
            let internal_channel = {
                let mut config = <#device_type_path as ::lokey::Device>::internal_channel_config();
                modify_internal_channel_config(&mut config);
                let channel_impl = ::lokey::internal::ChannelConfig::init(
                    config,
                    mcu,
                    spawner
                ).await;
                ::lokey::internal::Channel::new(channel_impl, spawner)
            };

            let external_channel = {
                let mut config = <#device_type_path as ::lokey::Device>::external_channel_config();
                modify_external_channel_config(&mut config);
                let channel_impl = ::lokey::external::ChannelConfig::init(
                    config,
                    mcu,
                    spawner,
                    internal_channel.as_dyn()
                ).await;
                ::lokey::external::Channel::new(channel_impl)
            };

            ::lokey::mcu::McuInit::run(mcu, spawner);

            let context = ::lokey::Context {
                spawner,
                mcu,
                external_channel,
                internal_channel,
                layer_manager: ::lokey::LayerManager::new(),
            };

            #function

            #function_ident(context).await
        }
    }
    .into()
}

#[proc_macro_error]
#[proc_macro]
pub fn layout(item: TokenStream) -> TokenStream {
    let arrays: syn::punctuated::Punctuated<syn::ExprArray, syn::token::Comma> =
        syn::punctuated::Punctuated::parse_terminated
            .parse(item)
            .unwrap_or_else(|e| abort!("{}", e.to_string()));

    let num_keys = match arrays.first() {
        Some(v) => v.elems.len(),
        None => 0,
    };
    for array in &arrays {
        if array.elems.len() != num_keys {
            abort!(
                array.span(),
                "All layers must have an equal amount of actions"
            );
        }
    }
    let mut layer_actions: Vec<Vec<proc_macro2::TokenStream>> = vec![vec![]; num_keys];
    for array in arrays {
        for (key_index, expr) in array.elems.into_iter().enumerate() {
            let expr = match expr {
                syn::Expr::Path(path)
                    if path.path.get_ident().map(|v| v.to_string())
                        == Some("Transparent".to_owned()) =>
                {
                    match layer_actions[key_index].last() {
                        Some(v) => v.clone(),
                        None => quote! { ::lokey::key::action::NoOp },
                    }
                }
                _ => expr.to_token_stream(),
            };
            layer_actions[key_index].push(expr);
        }
    }
    let combined_actions = layer_actions
        .into_iter()
        .map(|actions| {
            let v = actions
                .into_iter()
                .enumerate()
                .map(|(layer_index, action)| {
                    let layer_index = u8::try_from(layer_index).unwrap();
                    quote! {{
                        type A = impl ::lokey::key::DynAction;
                        // This `AtomicBool` is only used here to make the static not implement
                        // `core::marker::Freeze`. Without it the compiler has to look up wheter `T`
                        // implements `Freeze` which results in a cycle for the trait lookup,
                        // causing a compilation error.
                        static ACTION: (A, ::core::sync::atomic::AtomicBool) =
                            (#action, ::core::sync::atomic::AtomicBool::new(false));
                        (::lokey::LayerId(#layer_index), &ACTION.0)
                    }}
                })
                .collect::<Vec<_>>();
            let num_actions = v.len();
            quote! {{
                static PER_LAYER_ACTION: ::lokey::key::action::PerLayer<#num_actions> =
                    ::lokey::key::action::PerLayer::new([#(#v,)*]);
                &PER_LAYER_ACTION
            }}
        })
        .collect::<Vec<_>>();
    quote! {
        ::lokey::key::Layout::new([#(#combined_actions,)*])
    }
    .into()
}
