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
                        None => quote! { ::lokey_keyboard::action::NoOp },
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
                    quote! {{
                        trait ConstructAction {
                            type Type;
                            fn construct() -> Self::Type;
                        }
                        impl ConstructAction for () {
                            type Type = impl ::lokey_keyboard::Action;
                            fn construct() -> Self::Type {
                                #action
                            }
                        }
                        static ACTION: ::lokey::static_cell::StaticCell<<() as ConstructAction>::Type> = ::lokey::static_cell::StaticCell::new();
                        let action = ::lokey_keyboard::DynAction::from_ref(ACTION.init(<() as ConstructAction>::construct()));
                        (::lokey_common::layer::LayerId(#layer_index), action)
                    }}
                })
                .collect::<Vec<_>>();
            let num_actions = v.len();
            quote! {{
                static PER_LAYER_ACTION: ::lokey::static_cell::StaticCell<::lokey_keyboard::action::PerLayer<#num_actions>> = ::lokey::static_cell::StaticCell::new();
                let action = PER_LAYER_ACTION.init(::lokey_keyboard::action::PerLayer::new([#(#v,)*]));
                ::lokey_keyboard::DynAction::from_ref(action)
            }}
        })
        .collect::<Vec<_>>();
    let num_actions = combined_actions.len();
    quote! {{
        static LAYOUT: ::lokey::static_cell::StaticCell<::lokey_keyboard::Layout<#num_actions>> = ::lokey::static_cell::StaticCell::new();
        LAYOUT.init(::lokey_keyboard::Layout::new([#(#combined_actions,)*]))
    }}
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
                        static DYN_ACTION: &'static ::lokey_keyboard::DynAction = ::lokey_keyboard::DynAction::from_ref(&#action);
                        (::lokey_common::layer::LayerId(#layer_index), DYN_ACTION)
                    }}
                })
                .collect::<Vec<_>>();
            let num_actions = v.len();
            quote! {{
                static PER_LAYER_ACTION: ::lokey_keyboard::action::PerLayer<#num_actions> =
                    ::lokey_keyboard::action::PerLayer::new([#(#v,)*]);
                ::lokey_keyboard::DynAction::from_ref(&PER_LAYER_ACTION)
            }}
        })
        .collect::<Vec<_>>();
    quote! {
        ::lokey_keyboard::Layout::new([#(#combined_actions,)*])
    }
    .into()
}
