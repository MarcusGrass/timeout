use crate::Error;
#[cfg(not(feature = "test"))]
use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
#[cfg(feature = "test")]
use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use std::time::Duration;

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
                let mut inner_group = TokenStream::new();
                inner_group.extend([
                    TokenTree::Literal(Literal::u64_unsuffixed(secs)),
                    TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                    TokenTree::Literal(Literal::u32_suffixed(nanos)),
                ]);
                let mut ts = TokenStream::new();
                let span = Span::call_site();
                ts.extend([
                    TokenTree::Ident(Ident::new("core", span)),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("time", span)),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("Duration", span)),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("new", span)),
                    TokenTree::Group(Group::new(Delimiter::Parenthesis, inner_group)),
                ]);
                ts
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
            OnError::Panic => {
                let mut group = TokenStream::new();
                group.extend([TokenTree::Literal(Literal::string("timeout"))]);
                let mut ts = TokenStream::new();
                let span = Span::call_site();
                ts.extend([
                    TokenTree::Ident(Ident::new("panic", span)),
                    TokenTree::Punct(Punct::new('!', Spacing::Alone)),
                    TokenTree::Group(Group::new(Delimiter::Parenthesis, group)),
                ]);
                ts
            }
            OnError::Result(e) => {
                let mut inner_group = TokenStream::new();
                inner_group.extend([TokenTree::Literal(Literal::string("timeout"))]);
                let mut outer_group = TokenStream::new();
                outer_group.extend(e);
                outer_group.extend([TokenTree::Group(Group::new(
                    Delimiter::Parenthesis,
                    inner_group,
                ))]);
                let mut ts = TokenStream::new();
                ts.extend([
                    TokenTree::Ident(Ident::new("Err", Span::call_site())),
                    TokenTree::Group(Group::new(Delimiter::Parenthesis, outer_group)),
                ]);
                ts
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
                return Err(Error::with_span(
                    id.span(),
                    "Duplicate 'duration' attribute",
                ));
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
        return Err(Error::with_span(
            p.span(),
            format!("Expected '=' after '{}', found '{}'", attr, p),
        ));
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
                stream.extend([next]);
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
                stream.extend([next]);
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
        match &next {
            TokenTree::Literal(lit) => {
                let lit_s = lit.to_string();
                let lit_s = lit_s.trim_matches('"');
                return if lit_s == "panic" {
                    Ok(OnError::Panic)
                } else {
                    Err(Error::with_span(
                        lit.span(),
                        format!("Got 'on_error' str literal, expected only 'panic', got {lit}"),
                    ))
                };
            }
            TokenTree::Ident(_id) => {
                stream.extend([next]);
            }
            TokenTree::Punct(p) => {
                if p.as_char() == ',' {
                    break Ok(OnError::Result(stream));
                }
                stream.extend([next]);
            }
            t => {
                return Err(Error::with_span(
                    t.span(),
                    format!("Expected 'on_error' str literal or ident, got '{}'", t),
                ));
            }
        }
        if let Some(n) = it.next() {
            next = n;
        } else {
            break Ok(OnError::Result(stream));
        }
    }
}
