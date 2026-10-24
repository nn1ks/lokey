mod tx_message;

use proc_macro::TokenStream;

#[proc_macro_derive(TxMessage)]
pub fn tx_message_derive(item: TokenStream) -> TokenStream {
    tx_message::tx_message_derive(item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
