extern crate proc_macro;

use crate::compile_error::to_compile_error;
use crate::inject::{try_inject, Injector};
use crate::parse_attr::{parse_attr, ValidOpts};
use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use std::fmt::Display;

mod compile_error;
pub mod inject;
pub mod parse_attr;
pub mod parse_duration;

struct TokioTimeoutInjector(ValidOpts);

pub(crate) enum Error {
    Parse(Span, String),
    ParseSpanMissing(String),
}

impl Error {
    pub fn missing_span(msg: String) -> Self {
        Self::ParseSpanMissing(msg)
    }

    pub fn with_span<T: Display>(span: Span, msg: T) -> Self {
        Self::Parse(span, msg.to_string())
    }

    pub fn with_span_if_missing(self, span: Span) -> Self {
        match self {
            Self::Parse(_, _) => self,
            Self::ParseSpanMissing(e) => Self::Parse(span, e),
        }
    }

    pub fn into_token_stream_with_fallback_span(self, span: Span) -> TokenStream {
        match self {
            Self::Parse(p, msg) => to_compile_error(&msg, p),
            Self::ParseSpanMissing(e) => to_compile_error(&e, span),
        }
    }
}

pub(crate) type Result<T> = core::result::Result<T, Error>;

impl Injector for TokioTimeoutInjector {
    fn inject(self, inner_code: TokenStream) -> TokenStream {
        let dur = self.0.duration.into_token_stream();
        let on_timeout = self.0.on_error.into_token_stream();
        let mut inner = TokenStream::new();
        let span = Span::call_site();
        let mut timeout_args = TokenStream::new();
        timeout_args.extend(dur);
        timeout_args.extend([
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(Ident::new("async", span)),
            TokenTree::Group(Group::new(Delimiter::Brace, inner_code)),
        ]);
        let mut match_body = TokenStream::new();
        let mut value_group = TokenStream::new();
        value_group.extend([TokenTree::Ident(Ident::new("v", span))]);
        let mut err_group = TokenStream::new();
        err_group.extend([TokenTree::Ident(Ident::new("e", span))]);
        match_body.extend([
            TokenTree::Ident(Ident::new("Ok", span)),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, value_group)),
            TokenTree::Punct(Punct::new('=', Spacing::Joint)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Ident(Ident::new("v", span)),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(Ident::new("Err", span)),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, err_group)),
            TokenTree::Punct(Punct::new('=', Spacing::Joint)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
        ]);
        match_body.extend(on_timeout);
        inner.extend([
            TokenTree::Ident(Ident::new("match", span)),
            TokenTree::Ident(Ident::new("tokio", span)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("time", span)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("timeout", span)),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, timeout_args)),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("await", span)),
            TokenTree::Group(Group::new(Delimiter::Brace, match_body)),
        ]);
        TokenStream::from(TokenTree::Group(Group::new(Delimiter::Brace, inner)))
    }
}

pub fn tokio_timeout(attr: TokenStream, item: TokenStream) -> TokenStream {
    let validated = match parse_attr(attr) {
        Ok(o) => o,
        Err(e) => {
            return e.into_token_stream_with_fallback_span(Span::call_site());
        }
    };
    try_inject(TokioTimeoutInjector(validated), item)
        .unwrap_or_else(|e| e.into_token_stream_with_fallback_span(Span::call_site()))
}
