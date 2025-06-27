# This project is in no way associated with the tokio project

You can, at the time of writing, find that project [here](https://github.com/tokio-rs)

---

# tokio-timeout

[![CI][ci-badge]][ci-url]
[![Crates.io][crates-badge]][crates-url]
[![docs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

[ci-badge]: https://github.com/MarcusGrass/timeout/actions/workflows/check.yml/badge.svg?branch=main

[ci-url]: https://github.com/MarcusGrass/timeout/actions/workflows/check.yml

[crates-badge]: https://img.shields.io/crates/v/tokio-timeout.svg

[crates-url]: https://crates.io/crates/tokio-timeout

[docs-badge]: https://img.shields.io/docsrs/tokio-timeout/latest

[docs-url]: https://docs.rs/tokio-timeout/latest/tokio_timeout

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg

[mit-url]: https://github.com/MarcusGrass/tokio-timeout/blob/master/LICENSE


A proc macro attribute that can be put on an async function, running within a
tokio runtime, with the feature `time` enabled, which wraps the function in `tokio::time::timeout`.

```rust
#[tokio_timeout::timeout(duration = "1s", on_error = "panic")]
async fn my_fn() {
    println!("hello!");
}
```

## Why

In most of the asynchronous code that I write, a function has an upper limit of time
that it's supposed to run, it's just not codified. From http-requests, to reading/writing from disk,
awaiting a message from a channel, or whatever else. Not having a timeout is just hoping for the best,
which isn't particularly robust.

Adding timeouts to functions can be fairly cumbersome. If wanting to add timeout to all asynchronous calls in the code
base,
that requires restructuring blocks and matching on the potential timeout-case for each of the functions.

That's the reason that I wrote this minimal wrapping proc-macro.

## Usage

The macro takes two mandatory arguments 'duration' and 'on_error'.

### Duration

'Duration' can be either a string-literal that specifies a duration,
valid values are `<n>h` for hours, `<n>m` for minutes, `<n>s` for seconds, and `<n>ms`
for milliseconds. They can be chained together.

```rust
#[tokio_timeout::timeout(duration = "5h4m3s2ms", on_error = "panic")]
async fn my_fn() {
    println!("hello!");
}
```

Duration can also be specified to be some constant

```rust
use std::time::Duration;

const MY_DUR: Duration = Duration::from_millis(55);

#[tokio_timeout::timeout(duration = MY_DUR, on_error = "panic")]
async fn my_fn() {
    println!("hello!");
}
```

### On error

On error can either be the string literal "panic", as seen in examples above,
or something that can be invoked with a `&'static str` to produce an error.

```rust

fn to_error_result(s: &str) -> Result<(), String> {
    Err(s.to_string())
}

#[tokio_timeout::timeout(duration = "5h4m3s2ms", on_error = to_error_result)]
async fn my_fn_string_err() -> Result<(), String> {
    println!("hello!");
    Ok(())
}

pub enum MyErr {
    Timeout(&'static str)
}

const fn to_error_enum(s: &'static str) -> Result<(), MyErr> {
    Err(MyErr::Timeout(s))
}

#[tokio_timeout::timeout(duration = "5h4m3s2ms", on_error = to_error_enum)]
async fn my_fn_enum_err() -> Result<(), MyErr> {
    println!("hello!");
    Ok(())
}

fn print_err(s: &'static str) {
    eprintln!("oh no: {s}")
}

#[tokio_timeout::timeout(duration = "5h4m3s2ms", on_error = print_err)]
async fn my_print_timeout_fn() {
    println!("hello!");
}

#[tokio_timeout::timeout(duration = "5h4m3s2ms", on_error = anyhow::bail!)]
async fn anyhow_err_fn() -> anyhow::Result<()> {
    println!("hello!");
    Ok(())
}

```

## Goals

There are two goals that this crate aims to achieve additional to what the name states.

#### Low Compilation time

Since the functionality is so trivial, and the scope is theoretically any asynchronous function in a project
(any asynchronous functions that are not meant to run infinitely long, blocking progress forever), the primary
goal of the proc-macro is to compile quickly.
Additionally, it should barely add any compilation overhead, no matter how much it's used.

#### Informative

When something times out, it should be as obvious as possible to figure out what it was.

The macro will print the function name of the function that timed out, as well as the set timeout duration, if
the macro can statically determine that.

## Details

What the macro does is trivial, it takes some function:

```rust
#[tokio_timeout::timeout(duration = "1s", on_error = "panic")]
async fn my_fun() {}
```

and transforms it to this:

```rust
async fn my_fun() {
    match tokio::time::timeout(core::time::Duration::new(1, 0), async {}).await {
        Ok(o) => o,
        Err(_e) => panic!("...") // Displays where it timed out and if possible, how long the duration was
    }
}
```

That means it can be implemented without the `syn` + `quote` + `proc-macro2` stack.
This means that compilation time is kept down significantly if those are not already part of the project (which
they will be if the feature `macros` of `tokio` is enabled).

Benchmarking also shows that they add a significant (relative) overhead to compilation on each function
that has the attribute added to it.

### Some cursory benchmarking results

Using syn to parse the token stream, and quote to generate the output: `~13.5μs`.  
Parsing manually and using quote to generate the output: `~3.5μs`.  
Parsing manually and generating output manually: `~2.35μs`.  
Removing syn and quote completely (still using proc-macro2): `~1.69μs`.  
Removing proc-macro2: `???` (can't benchmark without using it because the library cannot be exported).

Compilation-overhead caused by the macro is reduced by `9/10` if the `syn` + `quote` + `proc-macro2` stack is not used.
Or, about `12μs` per function with the attribute.

These results mean that if the macro adds an overhead of `2μs`, it will cause an increase in compilation time
by `1ms` if used on 500 functions.

# License

This project is licensed under the MIT license, it can be found [here](./LICENSE).

Additionally, some code was copied from `syn`, you can find that code and a link to the original code and license
at the top of the file [here](./timeout-macro-parse/src/compile_error.rs).  
