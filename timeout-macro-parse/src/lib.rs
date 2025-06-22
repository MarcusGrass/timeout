use crate::inject::{try_inject, Injector};
use crate::parse_attr::{parse_attr, OnError, ParsedDuration, ValidOpts};
use proc_macro2::TokenStream;

pub mod inject;
pub mod parse_attr;
pub mod parse_duration;

struct TokioTimeoutInjector(ValidOpts);

impl Injector for TokioTimeoutInjector {
    fn inject(self, inner_code: TokenStream) -> Result<TokenStream, String> {
        let dur = match self.0.duration {
            ParsedDuration::Duration(d) => {
                let secs = d.as_secs();
                let nanos = d.subsec_nanos();
                quote::quote! {core::time::Duration::new(#secs, #nanos)}
            }
            ParsedDuration::Ref(r) => r,
        };
        let on_timeout = match self.0.on_error {
            OnError::Panic => quote::quote! {panic!("timeout") },
            OnError::Result(e) => {
                let raw = e.to_string();
                let raw = raw.trim_matches('"');
                let raw = syn::parse_str::<syn::Expr>(raw).map_err(|e| e.to_string())?;
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
        })
    }
}

pub fn tokio_timeout(attr: TokenStream, item: TokenStream) -> TokenStream {
    let validated = parse_attr(attr).unwrap();
    try_inject(TokioTimeoutInjector(validated), item).unwrap()
}
