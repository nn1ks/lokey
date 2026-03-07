use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Ident, parse_macro_input};

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

    let struct_generics = (0..combined_actions.len())
        .map(|i| Ident::new(&format!("A{}", i), proc_macro2::Span::call_site()))
        .collect::<Vec<_>>();
    let struct_definition = quote! {
        struct __LayoutActionContainer<#(#struct_generics),*>(
            #(#struct_generics,)*
        );
    };

    let num_children_typenum = Ident::new(
        &format!("U{}", combined_actions.len()),
        proc_macro2::Span::call_site(),
    );
    let field_indices = (0..combined_actions.len())
        .map(syn::Index::from)
        .collect::<Vec<_>>();
    let struct_impl = quote! {
        impl<#(#struct_generics: ::lokey_keyboard::Action),*> ::lokey_keyboard::ActionContainer for __LayoutActionContainer<#(#struct_generics),*> {
            type NumChildren = ::lokey_keyboard::typenum::#num_children_typenum;

            async fn child_on_press<D, T, S>(
                &self,
                child_index: usize,
                context: ::lokey::Context<D, T, S>,
            ) -> ::core::result::Result<(), ::lokey_keyboard::action::InvalidChildActionIndex>
            where
                D: ::lokey::Device,
                T: ::lokey::Transports<D::Mcu>,
                S: ::lokey::StateContainer
            {
                match child_index {
                    #(#field_indices => {
                        self.#field_indices.on_press(context).await;
                        ::core::result::Result::Ok(())
                    })*
                    _ => ::core::result::Result::Err(::lokey_keyboard::action::InvalidChildActionIndex { index: child_index })
                }
            }

            async fn child_on_release<D, T, S>(
                &self,
                child_index: usize,
                context: ::lokey::Context<D, T, S>,
            ) -> ::core::result::Result<(), ::lokey_keyboard::action::InvalidChildActionIndex>
            where
                D: ::lokey::Device,
                T: ::lokey::Transports<D::Mcu>,
                S: ::lokey::StateContainer
            {
                match child_index {
                    #(#field_indices => {
                        self.#field_indices.on_release(context).await;
                        ::core::result::Result::Ok(())
                    })*
                    _ => ::core::result::Result::Err(::lokey_keyboard::action::InvalidChildActionIndex { index: child_index })
                }
            }
        }
    };

    quote! {{
        #struct_definition
        #struct_impl
        ::lokey_keyboard::Layout::new(__LayoutActionContainer(#(#combined_actions,)*))
    }}
    .into()
}
