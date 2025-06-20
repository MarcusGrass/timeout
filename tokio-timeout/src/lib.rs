#![allow(dead_code, unused)]

use timeout_macro_parse::parse_attr::{parse_attr, OnError, ParsedDuration, ValidOpts};
use proc_macro::{Delimiter, Group, TokenStream};
use timeout_macro_parse::inject::{try_inject, Injector};

struct TimeoutInjector(ValidOpts);

impl Injector for TimeoutInjector {
    fn inject(self, inner_code: String) -> String {
        let dur = match self.0.duration {
            ParsedDuration::Duration(d) => {
                format!("core::time::Duration::new({}, {})", d.as_secs(), d.subsec_nanos())
            }
            ParsedDuration::Ref(r) => {
                r.to_string()
            }
        };
        let on_timeout = match self.0.on_error {
            OnError::Panic => "panic!(\"timeout\")".to_string(),
            OnError::Result(e) => format!("Err({e}(\"timeout\"))"),
        };

        format!("{{ match tokio::time::timeout( {dur}, async {{ {inner_code} }} ).await {{ Ok(v) => v, Err(e) => {on_timeout} }} }}")
    }
}

#[proc_macro_attribute]
pub fn timeout(attr: TokenStream, item: TokenStream) -> TokenStream {
    let validated = parse_attr(attr).unwrap();
    try_inject(TimeoutInjector(validated), item).unwrap()
}
