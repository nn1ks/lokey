mod layout;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

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
