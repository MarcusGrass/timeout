use crate::inject::{try_inject, Injector};
use crate::parse_attr::{parse_attr, ValidOpts};
use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};
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
