use crate::Error;
use proc_macro2::{TokenStream, TokenTree};
use quote::TokenStreamExt;
use std::time::Duration;
use syn::spanned::Spanned;

pub(crate) fn parse_attr(attr: TokenStream) -> crate::Result<ValidOpts> {
    let mut opts = Opts::default();
    let mut it = attr.into_iter();
    while take_next(&mut opts, &mut it)? {}
    Ok(ValidOpts {
        duration: opts
            .duration
            .ok_or_else(|| Error::missing_span("Missing 'duration' attribute".to_string()))?,
        on_error: opts
            .on_error
            .ok_or_else(|| Error::missing_span("Missing 'on_error'".to_string()))?,
    })
}

pub struct ValidOpts {
    pub duration: ParsedDuration,
    pub on_error: OnError,
}

#[derive(Default)]
struct Opts {
    duration: Option<ParsedDuration>,
    on_error: Option<OnError>,
}

pub enum ParsedDuration {
    Duration(Duration),
    Ref(TokenStream),
}

impl ParsedDuration {
    pub fn into_token_stream(self) -> TokenStream {
        match self {
            ParsedDuration::Duration(d) => {
                let secs = d.as_secs();
                let nanos = d.subsec_nanos();
                quote::quote! {core::time::Duration::new(#secs, #nanos)}
            }
            ParsedDuration::Ref(r) => r,
        }
    }
}

pub enum OnError {
    Panic,
    Result(TokenStream),
}

impl OnError {
    pub fn into_token_stream(self) -> TokenStream {
        match self {
            OnError::Panic => quote::quote! {panic!("timeout") },
            OnError::Result(e) => {
                quote::quote! { Err(#e("timeout"))}
            }
        }
    }
}

enum Attributes {
    Duration,
    OnError,
}

fn take_next(cur: &mut Opts, it: &mut impl Iterator<Item = TokenTree>) -> crate::Result<bool> {
    let (attrs, id) = loop {
        let Some(next) = it.next() else {
            return Ok(false);
        };
        match next {
            TokenTree::Ident(id) => {
                break match id.to_string().as_str() {
                    "duration" => (Attributes::Duration, id),
                    "on_error" => (Attributes::OnError, id),
                    unk => {
                        return Err(Error::with_span(
                            id.span(),
                            format!("Unknown attribute: {}", unk),
                        ));
                    }
                };
            }
            // Allow a punct from the last round
            TokenTree::Punct(p) => {
                if p.as_char() != ',' {
                    return Err(Error::with_span(
                        p.span(),
                        format!("Only punctuation expected is comma, got '{p}'"),
                    ));
                }
            }
            t => {
                return Err(Error::with_span(
                    t.span(),
                    format!("Unexpected token: '{}'", t),
                ));
            }
        }
    };
    match attrs {
        Attributes::Duration => {
            if cur.duration.is_some() {
                return Err(Error::Parse(syn::Error::new(
                    id.span(),
                    "Duplicate 'duration' attribute",
                )));
            }
            take_next_equals(it, "duration").map_err(|e| e.with_span_if_missing(id.span()))?;
            cur.duration = Some(parse_duration(it)?);
        }
        Attributes::OnError => {
            if cur.on_error.is_some() {
                return Err(Error::with_span(
                    id.span(),
                    "Duplicate 'on_error' attribute",
                ));
            }
            take_next_equals(it, "on_error").map_err(|e| e.with_span_if_missing(id.span()))?;
            cur.on_error = Some(parse_on_error(it).map_err(|e| e.with_span_if_missing(id.span()))?);
        }
    }

    Ok(true)
}

fn take_next_equals(
    it: &mut impl Iterator<Item = TokenTree>,
    attr: &'static str,
) -> crate::Result<()> {
    let Some(next) = it.next() else {
        return Err(Error::missing_span(format!(
            "Expected '=' after '{}', found nothing",
            attr
        )));
    };
    let TokenTree::Punct(p) = next else {
        return Err(Error::with_span(
            next.span(),
            format!("Expected '=' after '{}', found '{}'", attr, next),
        ));
    };
    if p.as_char() != '=' {
        return Err(crate::Error::Parse(syn::Error::new(
            p.span(),
            format!("Expected '=' after '{}', found '{}'", attr, p),
        )));
    }
    Ok(())
}

fn parse_duration(it: &mut impl Iterator<Item = TokenTree>) -> crate::Result<ParsedDuration> {
    let Some(mut next) = it.next() else {
        return Err(Error::missing_span(
            "Expected duration token, got nothing".to_string(),
        ));
    };
    let mut stream = TokenStream::new();
    loop {
        match &next {
            TokenTree::Ident(_id) => {
                stream.append(next);
            }
            TokenTree::Literal(lit) => {
                return Ok(ParsedDuration::Duration(
                    crate::parse_duration::parse_duration(lit.to_string().as_str())
                        .map_err(|e| Error::with_span(lit.span(), e))?,
                ))
            }
            TokenTree::Punct(p) => {
                if p.as_char() == ',' {
                    break Ok(ParsedDuration::Ref(stream));
                }
                stream.append(p.clone());
            }
            t => {
                return Err(Error::with_span(
                    t.span(),
                    format!("Expected duration literal or ident, got '{}'", t),
                ))
            }
        }
        if let Some(n) = it.next() {
            next = n;
        } else {
            break Ok(ParsedDuration::Ref(stream));
        }
    }
}

fn parse_on_error(it: &mut impl Iterator<Item = TokenTree>) -> crate::Result<OnError> {
    let Some(mut next) = it.next() else {
        return Err(Error::ParseSpanMissing(
            "Expected 'on_error' token, got nothing".to_string(),
        ));
    };
    let mut stream = TokenStream::new();
    loop {
        match next {
            TokenTree::Literal(lit) => {
                let lit = lit.to_string();
                let lit = lit.trim_matches('"');
                return if lit == "panic" {
                    Ok(OnError::Panic)
                } else {
                    Err(Error::Parse(syn::Error::new(
                        lit.span(),
                        format!("Got 'on_error' str literal, expected only 'panic', got {lit}"),
                    )))
                };
            }
            TokenTree::Ident(id) => {
                stream.append(id);
            }
            TokenTree::Punct(p) => {
                if p.as_char() == ',' {
                    break Ok(OnError::Result(stream));
                }
                stream.append(p);
            }
            t => {
                return Err(Error::Parse(syn::Error::new(
                    t.span(),
                    format!("Expected 'on_error' str literal or ident, got '{}'", t),
                )));
            }
        }
        if let Some(n) = it.next() {
            next = n;
        } else {
            break Ok(OnError::Result(stream));
        }
    }
}
