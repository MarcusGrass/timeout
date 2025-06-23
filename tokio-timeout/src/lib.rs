use proc_macro::TokenStream;

/// # Timeout macro
///
/// A proc macro attribute that can be put on an async function, running within a
/// tokio runtime, which wraps the function in `tokio::time::timeout`
///
/// ```
/// #[tokio_timeout::timeout(duration = "1s", on_error = "panic")]
/// async fn my_fn() {
///     println!("hello!");
/// }
/// ```
///
/// It takes two mandatory arguments 'duration' and 'on_error'.
///
/// ## Duration
///
/// 'Duration' can be either a string-literal that specifies a duration,
/// valid values are `<n>h` for hours, `<n>m` for minutes, `<n>s` for seconds, and `<n>ms`
/// for milliseconds. They can be chained together.
///
/// ```
/// #[tokio_timeout::timeout(duration = "5h4m3s2ms", on_error = "panic")]
/// async fn my_fn() {
///     println!("hello!");
/// }
/// ```
///
/// Duration can also be specified to be some constant
///
/// ```
/// use std::time::Duration;
///
/// const MY_DUR: Duration = Duration::from_millis(55);
///
/// #[tokio_timeout::timeout(duration = MY_DUR, on_error = "panic")]
/// async fn my_fn() {
///     println!("hello!");
/// }
/// ```
///
/// ## On error
///
/// On error can either be the string literal "panic", as seen in examples above,
/// or something that can be invoked with a `&'static str` to produce an error.
///
/// ```
/// fn print_err(s: &str) -> String{
///    s.to_string()
/// }
///
/// #[tokio_timeout::timeout(duration = "5h4m3s2ms", on_error = print_err)]
/// async fn my_fn_string_err() -> Result<(), String>{
///     println!("hello!");
///     Ok(())
/// }
///
/// pub enum MyErr {
///     Timeout(&'static str)
/// }
///
/// #[tokio_timeout::timeout(duration = "5h4m3s2ms", on_error = MyErr::Timeout)]
/// async fn my_fn_enum_err() -> Result<(), MyErr>{
///     println!("hello!");
///     Ok(())
/// }
/// ```
///
/// ```compile_fail
/// #[tokio_timeout::timeout]
/// async fn both_attrs_needed() {}
/// ```
///
/// ```compile_fail
/// #[tokio_timeout::timeout]
/// fn only_async_functions() {}
/// ```
///
/// ```compile_fail
/// #[tokio_timeout::timeout(duration = "1z", on_error = "panic")]
/// async fn unrecognized_duration() {}
/// ```
///
/// ```compile_fail
/// #[tokio_timeout::timeout(duration = "1s", on_error = "panico")]
/// async fn unrecognized_on_error() {}
/// ```
///
#[proc_macro_attribute]
pub fn timeout(attr: TokenStream, item: TokenStream) -> TokenStream {
    timeout_macro_parse::tokio_timeout(attr, item)
}
