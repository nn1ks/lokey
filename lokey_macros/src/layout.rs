use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::{ToTokens, quote};
use syn::parse::Parser;
use syn::parse_macro_input;
use syn::spanned::Spanned;

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
                            ::lokey::layer::LayerId(#layer_index),
                            ::lokey::key::DynAction::from_ref(::alloc::boxed::Box::leak(::alloc::boxed::Box::new(#action)))
                        )
                    }
                })
                .collect::<Vec<_>>();
            quote! {
                ::lokey::key::DynAction::from_ref(::alloc::boxed::Box::leak(::alloc::boxed::Box::new(
                    ::lokey::key::action::PerLayer::new([#(#v,)*])
                )))
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
                        (::lokey::layer::LayerId(#layer_index), ::lokey::key::DynAction::from_ref(&ACTION))
                    }}
                })
                .collect::<Vec<_>>();
            let num_actions = v.len();
            quote! {{
                static PER_LAYER_ACTION: ::lokey::key::action::PerLayer<#num_actions> =
                    ::lokey::key::action::PerLayer::new([#(#v,)*]);
                ::lokey::key::DynAction::from_ref(&PER_LAYER_ACTION)
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
