#![allow(dead_code, unused)]

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};
use timeout_macro_parse::inject::{Injector, try_inject};
use timeout_macro_parse::parse_attr::{OnError, ParsedDuration, ValidOpts, parse_attr};

struct TimeoutInjector(ValidOpts);

impl Injector for TimeoutInjector {
    fn inject(self, inner_code: proc_macro2::TokenStream) -> Result<TokenStream, String> {
        let dur = match self.0.duration {
            ParsedDuration::Duration(d) => {
                let secs = d.as_secs();
                let nanos = d.subsec_nanos();
                quote::quote! {core::time::Duration::new(#secs, #nanos)}
            }
            ParsedDuration::Ref(r) => TokenStream::from(TokenTree::Ident(r)).into(),
        };
        let on_timeout = match self.0.on_error {
            OnError::Panic => quote::quote! {panic!("timeout") },
            OnError::Result(e) => {
                let raw = e.to_string();
                let raw = raw.trim_matches('"');
                let raw = syn::parse_str::<syn::Expr>(&raw).map_err(|e| e.to_string())?;
                quote::quote! { Err(#raw("timeout"))}
            }
        };
        Ok(quote::quote! {
            {
                match tokio::time::timeout( #dur, async { #inner_code } ).await {
                    Ok(v) => v,
                    Err(e) => #on_timeout
                }
            }
        }
        .into())
    }
}

#[proc_macro_attribute]
pub fn timeout(attr: TokenStream, item: TokenStream) -> TokenStream {
    let validated = parse_attr(attr).unwrap();
    try_inject(TimeoutInjector(validated), item).unwrap()
}
