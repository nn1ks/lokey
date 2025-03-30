use darling::FromMeta;
use darling::ast::NestedMeta;
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{ToTokens, quote};
use syn::parse::Parser;
use syn::parse_macro_input;
use syn::spanned::Spanned;

#[derive(FromMeta)]
struct DeviceArgs {
    heap_size: Option<syn::Expr>,
    address: Option<syn::Expr>,
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
                &external_transport_config,
                &internal_transport_config,
                spawner
            );
            let mcu = ::alloc::boxed::Box::leak(::alloc::boxed::Box::new(mcu));

            // Create channels
            let internal_channel = {
                let transport = ::lokey::internal::TransportConfig::init(
                    internal_transport_config,
                    mcu,
                    address,
                    spawner
                ).await;
                let transport = ::alloc::boxed::Box::leak(::alloc::boxed::Box::new(transport));
                ::lokey::internal::Channel::new(transport, spawner)
            };

            let external_channel = {
                let transport = ::lokey::external::TransportConfig::init(
                    external_transport_config,
                    mcu,
                    address,
                    spawner,
                    internal_channel.as_dyn()
                ).await;
                let transport = ::alloc::boxed::Box::leak(::alloc::boxed::Box::new(transport));
                ::lokey::external::Channel::new(transport)
            };

            let context = ::lokey::Context {
                spawner,
                address,
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

fn layer_actions(
    arrays: syn::punctuated::Punctuated<syn::ExprArray, syn::token::Comma>,
) -> Vec<Vec<proc_macro2::TokenStream>> {
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
    layer_actions
}

#[proc_macro_error]
#[proc_macro]
pub fn layout(item: TokenStream) -> TokenStream {
    let arrays: syn::punctuated::Punctuated<syn::ExprArray, syn::token::Comma> =
        syn::punctuated::Punctuated::parse_terminated
            .parse(item)
            .unwrap_or_else(|e| abort!("{}", e.to_string()));

    let layer_actions = layer_actions(arrays);

    let combined_actions = layer_actions
        .into_iter()
        .map(|actions| {
            let v = actions
                .into_iter()
                .enumerate()
                .map(|(layer_index, action)| {
                    let layer_index = u8::try_from(layer_index).unwrap();
                    quote! {
                        (
                            ::lokey::LayerId(#layer_index),
                            ::alloc::boxed::Box::leak(::alloc::boxed::Box::new(#action))
                        )
                    }
                })
                .collect::<Vec<_>>();
            quote! {
                ::alloc::boxed::Box::leak(::alloc::boxed::Box::new(
                    ::lokey::key::action::PerLayer::new([#(#v,)*])
                ))
            }
        })
        .collect::<Vec<_>>();
    quote! {
        ::alloc::boxed::Box::leak(::alloc::boxed::Box::new(
            ::lokey::key::Layout::new([#(#combined_actions,)*])
        ))
    }
    .into()
}

struct StaticLayoutArguments {
    static_ident: syn::Ident,
    _comma: syn::Token![,],
    arrays: syn::punctuated::Punctuated<syn::ExprArray, syn::token::Comma>,
}

impl syn::parse::Parse for StaticLayoutArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            static_ident: input.parse()?,
            _comma: input.parse()?,
            arrays: syn::punctuated::Punctuated::parse_terminated(input)?,
        })
    }
}

#[proc_macro_error]
#[proc_macro]
pub fn static_layout(item: TokenStream) -> TokenStream {
    let arguments = parse_macro_input!(item as StaticLayoutArguments);
    let layer_actions = layer_actions(arguments.arrays);

    fn build_action_type_ident(key_index: usize, layer_index: usize) -> syn::Ident {
        syn::Ident::new(
            &format!("__Action_{key_index}_{layer_index}"),
            proc_macro2::Span::call_site(),
        )
    }

    let action_type_idents = layer_actions
        .iter()
        .enumerate()
        .flat_map(|(key_index, actions)| {
            actions
                .iter()
                .enumerate()
                .map(move |(layer_index, _)| build_action_type_ident(key_index, layer_index))
        })
        .collect::<Vec<_>>();
    let combined_actions = layer_actions
        .into_iter()
        .enumerate()
        .map(|(key_index, actions)| {
            let v = actions
                .into_iter()
                .enumerate()
                .map(|(layer_index, action)| {
                    let action_type_ident = build_action_type_ident(key_index, layer_index);
                    let layer_index = u8::try_from(layer_index).unwrap();
                    quote! {{
                        #[define_opaque(#action_type_ident)]
                        const fn action() -> #action_type_ident {
                            #action
                        }
                        static ACTION: #action_type_ident = action();
                        (::lokey::LayerId(#layer_index), &ACTION)
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
    let static_ident = arguments.static_ident;
    let num_keys = combined_actions.len();
    quote! {
        #(type #action_type_idents = impl ::lokey::key::Action;)*
        const fn __build_layout() -> ::lokey::key::Layout<#num_keys> {
            ::lokey::key::Layout::new([#(#combined_actions,)*])
        }
        static #static_ident: ::lokey::key::Layout<#num_keys> = __build_layout();
    }
    .into()
}
