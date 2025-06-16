use proc_macro::{Ident, TokenStream, TokenTree};
use std::time::Duration;

pub fn parse_attr(attr: TokenStream) -> Result<(), String> {
    let mut opts = Opts::default();
    let mut it = attr.into_iter();
    while take_next(&mut opts, &mut it)? {}
    Ok(())
}

#[derive(Default)]
struct Opts {
    duration: Option<ParsedDuration>,
    on_error: Option<String>,
}

enum ParsedDuration {
    Duration(Duration),
    Ref(Ident),
}

enum Attributes {
    Duration,
    OnError,
}

fn take_next(cur: &mut Opts, it: &mut impl Iterator<Item = TokenTree>) -> Result<bool, String>{
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
            take_next_equals(it, "duration")?;
            parse_duration(it)?;
        }
        Attributes::OnError => {
            take_next_equals(it, "on_error")?
        }
    }
    
    Ok(true)
}

fn take_next_equals(it: &mut impl Iterator<Item = TokenTree>, attr: &'static str) -> Result<(), String> {
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
        return Err("Expected duration literal, got nothing".to_string());
    };
    match next {
        TokenTree::Ident(id) => {
            Ok(ParsedDuration::Ref(id))
        }
        TokenTree::Literal(lit) => {
            Ok(ParsedDuration::Duration(crate::parse_duration::parse_duration(lit.to_string().as_str())?))
        }
        t => {
            Err(format!("Expected duration literal or ident, got '{}'", t))
        }
    }
}