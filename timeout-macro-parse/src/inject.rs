use crate::Error;
use proc_macro::{Delimiter, TokenStream, TokenTree};

pub trait Injector {
    fn inject(self, inner_code: TokenStream) -> TokenStream;
}

pub(crate) fn try_inject(
    injector: impl Injector,
    source: TokenStream,
) -> crate::Result<TokenStream> {
    let mut it = source.into_iter();
    let mut pre = TokenStream::new();
    let inner_body = extract_inner_body(&mut pre, &mut it)?;
    let res = injector.inject(inner_body);
    pre.extend([res]);
    Ok(pre)
}

fn extract_inner_body(
    pre: &mut TokenStream,
    source: &mut impl Iterator<Item = TokenTree>,
) -> crate::Result<TokenStream> {
    let mut seen_async = false;
    let mut seen_fn_decl = false;
    let mut last = None;
    let mut peek = source.peekable();
    while let Some(token) = peek.next() {
        match &token {
            TokenTree::Ident(id) => {
                let id = id.to_string();
                match id.as_str() {
                    "async" => seen_async = true,
                    "fn" => seen_fn_decl = true,
                    _ => {}
                }
            }
            t => {
                last = Some(t.clone());
            }
        }
        if peek.peek().is_some() {
            pre.extend([token]);
        }
    }
    if !seen_fn_decl {
        return Err(Error::missing_span(
            "'timeout' macro used on something without a 'fn' declaration".to_string(),
        ));
    }
    if !seen_async {
        return Err(Error::missing_span(
            "'timeout' macro only allowed on async functions".to_string(),
        ));
    }
    let Some(TokenTree::Group(group)) = last else {
        return Err(Error::missing_span(
            "'timeout' macro used on something without a body".to_string(),
        ));
    };
    if !matches!(group.delimiter(), Delimiter::Brace) {
        return Err(Error::with_span(
            group.span(),
            "'timeout' macro used on something without a body (last group not a brace)".to_string(),
        ));
    }
    Ok(group.stream())
}
