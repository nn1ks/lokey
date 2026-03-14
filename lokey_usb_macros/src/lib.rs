mod tx_message;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro_derive(TxMessage)]
pub fn tx_message_derive(item: TokenStream) -> TokenStream {
    tx_message::tx_message_derive(item)
}
