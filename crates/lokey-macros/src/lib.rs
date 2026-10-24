mod device;
mod external_message;
mod state;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn device(attr: TokenStream, item: TokenStream) -> TokenStream {
    device::device(attr, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(State, attributes(state))]
pub fn state_derive(item: TokenStream) -> TokenStream {
    state::state_derive(item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(ExternalMessage, attributes(external_message))]
pub fn external_message_derive(item: TokenStream) -> TokenStream {
    external_message::external_message_derive(item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
