use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::TokenStreamExt;

pub trait Injector {
    fn inject(self, inner_code: TokenStream) -> Result<TokenStream, String>;
}

pub fn try_inject(injector: impl Injector, source: TokenStream) -> Result<TokenStream, String> {
    let mut it = source.into_iter();
    let mut pre = TokenStream::new();
    let inner_body = extract_inner_body(&mut pre, &mut it)?;
    let res = injector.inject(inner_body)?;
    Ok(quote::quote! {
        #pre
        #res
    })
}

fn extract_inner_body(
    pre: &mut TokenStream,
    source: &mut impl Iterator<Item = TokenTree>,
) -> Result<TokenStream, String> {
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
            pre.append(token);
        }
    }
    if !seen_fn_decl {
        return Err("'timeout' macro used on something without a 'fn' declaration".to_string());
    }
    if !seen_async {
        return Err("'timeout' macro only allowed on async functions".to_string());
    }
    let Some(TokenTree::Group(group)) = last else {
        return Err("'timeout' macro used on something without a body".to_string());
    };
    assert!(
        matches!(group.delimiter(), Delimiter::Brace),
        "'timeout' macro used on something without a body (last group not a brace)"
    );
    Ok(group.stream())
}
