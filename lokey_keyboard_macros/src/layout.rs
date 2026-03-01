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
    let arrays = parse_macro_input!(
        item with Punctuated::<syn::ExprArray, syn::token::Comma>::parse_terminated
    );
    let layer_actions = layer_actions(arrays);

    let combined_actions = layer_actions
        .into_iter()
        .map(|actions| {
            let layer_indices = actions
                .iter()
                .enumerate()
                .map(|(i, _)| u8::try_from(i).unwrap())
                .collect::<Vec<_>>();
            let layer_ids = quote! {
                ::lokey_keyboard::generic_array::GenericArray::from_array(
                    [#(::lokey_keyboard::lokey_layer::LayerId(#layer_indices),)*]
                )
            };
            let actions = quote! { (#(#actions,)*) };
            quote! {
                ::lokey_keyboard::action::PerLayer::new(#actions, #layer_ids)
            }
        })
        .collect::<Vec<_>>();

    quote! {
        ::lokey_keyboard::Layout::new((#(#combined_actions,)*))
    }
    .into()
}
