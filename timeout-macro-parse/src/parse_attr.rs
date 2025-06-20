use proc_macro::{Ident, TokenStream, TokenTree};
use quote::ToTokens;
use std::time::Duration;

pub fn parse_attr(attr: TokenStream) -> Result<ValidOpts, String> {
    let mut opts = Opts::default();
    let mut it = attr.into_iter();
    while take_next(&mut opts, &mut it)? {}
    Ok(ValidOpts {
        duration: opts
            .duration
            .ok_or_else(|| "Missing 'duration' attribute".to_string())?,
        on_error: opts
            .on_error
            .ok_or_else(|| "Missing 'on_error'".to_string())?,
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
    Ref(Ident),
}

pub enum OnError {
    Panic,
    Result(proc_macro2::TokenStream),
}

enum Attributes {
    Duration,
    OnError,
}

fn take_next(cur: &mut Opts, it: &mut impl Iterator<Item = TokenTree>) -> Result<bool, String> {
    let id = loop {
        let Some(next) = it.next() else {
            return Ok(false);
        };
        match next {
            TokenTree::Ident(id) => {
                break match id.to_string().as_str() {
                    "duration" => Attributes::Duration,
                    "on_error" => Attributes::OnError,
                    unk => {
                        return Err(format!("Unknown attribute: {}", unk));
                    }
                };
            }
            // Allow a punct from the last round
            TokenTree::Punct(p) => {
                if p.as_char() != ',' {
                    return Err(format!("Only punctuation expected is comma, got '{p}'"));
                }
            }
            t => {
                return Err(format!("Unexpected token: '{}'", t));
            }
        }
    };
    match id {
        Attributes::Duration => {
            if cur.duration.is_some() {
                panic!("Duplicate 'duration' attribute");
            }
            take_next_equals(it, "duration")?;
            cur.duration = Some(parse_duration(it)?);
        }
        Attributes::OnError => {
            if cur.on_error.is_some() {
                panic!("Duplicate 'on_error' attribute");
            }
            take_next_equals(it, "on_error")?;
            cur.on_error = Some(parse_on_error(it)?);
        }
    }

    Ok(true)
}

fn take_next_equals(
    it: &mut impl Iterator<Item = TokenTree>,
    attr: &'static str,
) -> Result<(), String> {
    let Some(next) = it.next() else {
        return Err(format!("Expected '=' after '{}', found nothing", attr));
    };
    let TokenTree::Punct(p) = next else {
        return Err(format!("Expected '=' after '{}', found '{}'", attr, next));
    };
    if p.as_char() != '=' {
        return Err(format!("Expected '=' after '{}', found '{}'", attr, p));
    }
    Ok(())
}

fn parse_duration(it: &mut impl Iterator<Item = TokenTree>) -> Result<ParsedDuration, String> {
    let Some(next) = it.next() else {
        return Err("Expected duration token, got nothing".to_string());
    };
    match next {
        TokenTree::Ident(id) => Ok(ParsedDuration::Ref(id)),
        TokenTree::Literal(lit) => Ok(ParsedDuration::Duration(
            crate::parse_duration::parse_duration(lit.to_string().as_str())?,
        )),
        t => Err(format!("Expected duration literal or ident, got '{}'", t)),
    }
}

fn parse_on_error(it: &mut impl Iterator<Item = TokenTree>) -> Result<OnError, String> {
    let Some(next) = it.next() else {
        return Err("Expected 'on_error' token, got nothing".to_string());
    };
    match next {
        TokenTree::Literal(lit) => {
            let lit = lit.to_string();
            let lit = lit.trim_matches('"');
            if lit == "panic" {
                Ok(OnError::Panic)
            } else {
                Ok(OnError::Result(lit.to_token_stream()))
            }
        }
        t => Err(format!("Expected 'on_error' str literal, got '{}'", t)),
    }
}
