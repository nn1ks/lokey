mod device;
mod layout;
mod state;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn device(attr: TokenStream, item: TokenStream) -> TokenStream {
    device::device(attr, item)
}

#[proc_macro_error]
#[proc_macro]
pub fn layout(item: TokenStream) -> TokenStream {
    layout::layout(item)
}

#[proc_macro_error]
#[proc_macro]
pub fn static_layout(item: TokenStream) -> TokenStream {
    layout::static_layout(item)
}

#[proc_macro_error]
#[proc_macro_derive(State)]
pub fn state_derive(item: TokenStream) -> TokenStream {
    state::state_derive(item)
}
