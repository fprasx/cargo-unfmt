# cargo-unfmt

Unformat code into perfect rectangles!

Take control of your formatting and turn [this](https://github.com/rust-lang/rust/blob/master/compiler/rustc_codegen_cranelift/example/polymorphize_coroutine.rs)
convoluted code:

```rust
#![feature(coroutines, coroutine_trait)]

use std::ops::Coroutine;
use std::pin::Pin;

fn main() {
    run_coroutine::<i32>();
}

fn run_coroutine<T>() {
    let mut coroutine = || {
        yield;
        return;
    };
    Pin::new(&mut coroutine).resume(());
}
```

into this beautiful block:

```rust
#![feature(coroutines,coroutine_trait)]use std::ops::Coroutine;use std::pin:://;
Pin;fn main(){;run_coroutine::<i32>();;}fn run_coroutine<T>(){let mut coroutine=
||{{;};yield;{;};{;};return;{;};};();();Pin::new(&mut coroutine).resume(());();}
```

## Installation

Install using `cargo` with:
```
cargo install cargo-unfmt --locked
```

## How does it work?
Through a combination of lexical and syntactic analysis, `cargo-unfmt` inserts
no-op statements like `if false{}`, extra parenthesis around expressions, and
comments to achieve perfect blocks. It tries to minimize the size of the resulting
code as well as minimize the number of end of line comments.

## License

This code is licensed under the GPL version 3 or later, because rectangular code
is for everyone!

Copyright 2024-present Felix Prasanna
