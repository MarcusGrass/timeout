use crate::inject::{Injector, try_inject};
use crate::parse_attr::{OnError, ParsedDuration, ValidOpts, parse_attr};
use proc_macro2::TokenStream;

pub mod inject;
pub mod parse_attr;
pub mod parse_duration;

struct TokioTimeoutInjector(ValidOpts);

impl Injector for TokioTimeoutInjector {
    fn inject(self, inner_code: TokenStream) -> Result<TokenStream, String> {
        let dur = match self.0.duration {
            ParsedDuration::Duration(d) => {
                let secs = d.as_secs();
                let nanos = d.subsec_nanos();
                quote::quote! {core::time::Duration::new(#secs, #nanos)}
            }
            ParsedDuration::Ref(r) => r,
        };
        let on_timeout = match self.0.on_error {
            OnError::Panic => quote::quote! {panic!("timeout") },
            OnError::Result(e) => {
                let raw = e.to_string();
                let raw = raw.trim_matches('"');
                let raw = syn::parse_str::<syn::Expr>(raw).map_err(|e| e.to_string())?;
                quote::quote! { Err(#raw("timeout"))}
            }
        };
        Ok(quote::quote! {
            {
                match tokio::time::timeout( #dur, async { #inner_code } ).await {
                    Ok(v) => v,
                    Err(e) => #on_timeout
                }
            }
        })
    }
}

pub fn tokio_timeout(attr: TokenStream, item: TokenStream) -> TokenStream {
    let validated = parse_attr(attr).unwrap();
    try_inject(TokioTimeoutInjector(validated), item).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    const SMALL_RAW_FN: &str = r#"
        async fn add(first: i64, second: i64) -> i64 {
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            first + second
        }
        "#;
    #[test]
    fn mk_fns() {
        const SMALL_DUR_ATTR_WITH_PANIC: &str = r#"duration = "1ms", on_error="panic""#;
        const LONG_DUR_ATTR_WITH_PANIC: &str =
            r#"duration = "1h100ms20m10s25ms15s", on_error="panic""#;
        let short_panic = parse_raw_fn(SMALL_DUR_ATTR_WITH_PANIC);
        let long_panic = parse_raw_fn(LONG_DUR_ATTR_WITH_PANIC);
        eprintln!("{}", std::env::current_dir().unwrap().display());
        std::fs::write(
            "benches/example-files/short_dur_attr_panic.rs",
            short_panic.to_string(),
        )
        .unwrap();
        std::fs::write(
            "benches/example-files/long_dur_attr_panic.rs",
            long_panic.to_string(),
        )
        .unwrap();
    }
    fn parse_raw_fn(attr: &str) -> TokenStream {
        let attr = TokenStream::from_str(attr).unwrap();
        let input = TokenStream::from_str(SMALL_RAW_FN).unwrap();
        tokio_timeout(attr.into(), input.into())
    }
}
