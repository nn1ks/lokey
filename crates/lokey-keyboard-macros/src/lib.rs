mod layout;

use proc_macro::TokenStream;

#[proc_macro]
pub fn layout(item: TokenStream) -> TokenStream {
    layout::layout(item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
