use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::{ToTokens, quote};
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

fn layer_actions(
    arrays: Punctuated<syn::ExprArray, syn::token::Comma>,
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
                        None => quote! { ::lokey::keyboard::action::NoOp },
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
    let arrays = parse_macro_input!(item with Punctuated::<syn::ExprArray, syn::token::Comma>::parse_terminated);

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
                            ::lokey::keyboard::DynAction::from_ref(::alloc::boxed::Box::leak(::alloc::boxed::Box::new(#action)))
                        )
                    }
                })
                .collect::<Vec<_>>();
            quote! {
                ::lokey::keyboard::DynAction::from_ref(::alloc::boxed::Box::leak(::alloc::boxed::Box::new(
                    ::lokey::keyboard::action::PerLayer::new([#(#v,)*])
                )))
            }
        })
        .collect::<Vec<_>>();
    quote! {
        ::alloc::boxed::Box::leak(::alloc::boxed::Box::new(
            ::lokey::keyboard::Layout::new([#(#combined_actions,)*])
        ))
    }
    .into()
}

pub fn static_layout(item: TokenStream) -> TokenStream {
    let arrays = parse_macro_input!(item with Punctuated::<syn::ExprArray, syn::token::Comma>::parse_terminated);
    let layer_actions = layer_actions(arrays);

    let combined_actions = layer_actions
        .into_iter()
        .map(|actions| {
            let v = actions
                .into_iter()
                .enumerate()
                .map(|(layer_index, action)| {
                    let layer_index = u8::try_from(layer_index).unwrap();
                    quote! {{
                        static DYN_ACTION: &'static ::lokey::keyboard::DynAction = ::lokey::keyboard::DynAction::from_ref(&#action);
                        (::lokey::layer::LayerId(#layer_index), DYN_ACTION)
                    }}
                })
                .collect::<Vec<_>>();
            let num_actions = v.len();
            quote! {{
                static PER_LAYER_ACTION: ::lokey::keyboard::action::PerLayer<#num_actions> =
                    ::lokey::keyboard::action::PerLayer::new([#(#v,)*]);
                ::lokey::keyboard::DynAction::from_ref(&PER_LAYER_ACTION)
            }}
        })
        .collect::<Vec<_>>();
    quote! {
        ::lokey::keyboard::Layout::new([#(#combined_actions,)*])
    }
    .into()
}
