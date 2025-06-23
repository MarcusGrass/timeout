use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn timeout(attr: TokenStream, item: TokenStream) -> TokenStream {
    timeout_macro_parse::tokio_timeout(attr, item)
}
