#![allow(dead_code, unused)]
mod parse_attr;
mod parse_duration;

use proc_macro::TokenStream;
use crate::parse_attr::parse_attr;

#[proc_macro_attribute]
pub fn timeout(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_attr(attr).unwrap();
    item
}