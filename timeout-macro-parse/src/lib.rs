use crate::inject::{try_inject, Injector};
use crate::parse_attr::{parse_attr, ValidOpts};
use proc_macro2::TokenStream;
use std::fmt::Display;
use syn::spanned::Spanned;

pub mod inject;
pub mod parse_attr;
pub mod parse_duration;

struct TokioTimeoutInjector(ValidOpts);

pub(crate) enum Error {
    Parse(syn::Error),
    ParseSpanMissing(String),
}

impl Error {
    pub fn missing_span(msg: String) -> Self {
        Self::ParseSpanMissing(msg)
    }

    pub fn with_span<T: Display>(span: proc_macro2::Span, msg: T) -> Self {
        Self::Parse(syn::Error::new(span, msg))
    }

    pub fn with_span_if_missing(self, span: proc_macro2::Span) -> Self {
        match self {
            Self::Parse(ref _p) => self,
            Self::ParseSpanMissing(e) => Self::Parse(syn::Error::new(span, e)),
        }
    }

    pub fn into_to_syn_with_fallback_span(self, span: proc_macro2::Span) -> syn::Error {
        match self {
            Self::Parse(p) => p.clone(),
            Self::ParseSpanMissing(e) => syn::Error::new(span, e),
        }
    }
}

pub(crate) type Result<T> = core::result::Result<T, Error>;

impl Injector for TokioTimeoutInjector {
    fn inject(self, inner_code: TokenStream) -> TokenStream {
        let dur = self.0.duration.into_token_stream();
        let on_timeout = self.0.on_error.into_token_stream();
        quote::quote! {
            {
                match tokio::time::timeout( #dur, async { #inner_code } ).await {
                    Ok(v) => v,
                    Err(e) => #on_timeout
                }
            }
        }
    }
}

pub fn tokio_timeout(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_span = attr.span();
    let validated = match parse_attr(attr) {
        Ok(o) => o,
        Err(e) => {
            return e
                .into_to_syn_with_fallback_span(attr_span)
                .to_compile_error();
        }
    };
    let item_span = item.span();
    match try_inject(TokioTimeoutInjector(validated), item) {
        Ok(o) => o,
        Err(e) => e
            .into_to_syn_with_fallback_span(item_span)
            .to_compile_error(),
    }
}
