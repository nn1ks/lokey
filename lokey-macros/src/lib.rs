mod device;
mod external_message;
mod state;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn device(attr: TokenStream, item: TokenStream) -> TokenStream {
    device::device(attr, item)
}

#[proc_macro_error]
#[proc_macro_derive(State, attributes(state))]
pub fn state_derive(item: TokenStream) -> TokenStream {
    state::state_derive(item)
}

#[proc_macro_error]
#[proc_macro_derive(ExternalMessage)]
pub fn external_message_derive(item: TokenStream) -> TokenStream {
    external_message::external_message_derive(item)
}
