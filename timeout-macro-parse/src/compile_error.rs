//! Ripped from syn at `f2588dcf1b4be92817d6e920d4267e0a160bc618`
//! The file at that commit can be found at [github here](https://github.com/dtolnay/syn/blob/f2588dcf1b4be92817d6e920d4267e0a160bc618/src/error.rs)
//! Licensed under MIT, the license at the same commit can be found at
//! [github here](https://github.com/dtolnay/syn/blob/f2588dcf1b4be92817d6e920d4267e0a160bc618/LICENSE-MIT)
#[cfg(not(feature = "test"))]
use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
#[cfg(feature = "test")]
use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

pub(crate) fn to_compile_error(message: &str, span: Span) -> TokenStream {
    // ::core::compile_error!($message)
    let mut ts = TokenStream::new();
    ts.extend([
        TokenTree::Punct({
            let mut punct = Punct::new(':', Spacing::Joint);
            punct.set_span(span);
            punct
        }),
        TokenTree::Punct({
            let mut punct = Punct::new(':', Spacing::Alone);
            punct.set_span(span);
            punct
        }),
        TokenTree::Ident(Ident::new("core", span)),
        TokenTree::Punct({
            let mut punct = Punct::new(':', Spacing::Joint);
            punct.set_span(span);
            punct
        }),
        TokenTree::Punct({
            let mut punct = Punct::new(':', Spacing::Alone);
            punct.set_span(span);
            punct
        }),
        TokenTree::Ident(Ident::new("compile_error", span)),
        TokenTree::Punct({
            let mut punct = Punct::new('!', Spacing::Alone);
            punct.set_span(span);
            punct
        }),
        TokenTree::Group({
            let mut group = Group::new(Delimiter::Brace, {
                TokenStream::from_iter([TokenTree::Literal({
                    let mut string = Literal::string(message);
                    string.set_span(span);
                    string
                })])
            });
            group.set_span(span);
            group
        }),
    ]);
    ts
}
