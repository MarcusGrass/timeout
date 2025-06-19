#![allow(dead_code, unused)]
mod parse_attr;
mod parse_duration;

use crate::parse_attr::parse_attr;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn timeout(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_attr(attr).unwrap();
    item
}
