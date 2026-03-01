mod layout;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro]
pub fn layout(item: TokenStream) -> TokenStream {
    layout::layout(item)
}
