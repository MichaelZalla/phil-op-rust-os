# 1. A Freestanding Rust Binary

## Introduction

The [Rust standard library](https://doc.rust-lang.org/std/) provides programmers with common primitive types, traits, and standard Rust macros—as well as modules for interacting with operating system services, such as:
- Heap management
- Thread scheduling
- File and network I/O
- Secure random numbers

Consequently, the standard library requires that it be compiled and run in the context of a [supported operating system target](https://doc.rust-lang.org/nightly/rustc/platform-support.html) (such as x86 Windows). To use Rust to create a freestanding, or "baremetal" program—one that serves as its own operating system—we won't have access to `std` and its features.

We _will_, however, have access to a few Rust intrinsics provided in Rust's [`core` crate](https://doc.rust-lang.org/core/index.html). This includes:
- Basic functions to work with memory and pointers.
- Utilities for working with arrays, slices, strings, and iterators.
- Primitive type conversions.
- Simple support for handling ASCII and Unicode data.
- Core enums like `Option` and `Result`.
- Core traits like `Default`, `Sized`, `Copy`, `Send`, `Sync`, and `Error`.
- Core macros including `assert!()`, `cfg!()`, `env!()`, and `panic!()`, as well as `todo!()` and `!unimplemented()`.

utilities for arrays and iterators, type conversion, 

One common feature that will be off-limits, for the time being, is Rust's print macros (like `println!()`). These macros are defined in `std` and rely on standard output, which is a file descriptor.

## Disabling the Standard Library

We can use Rust's `no_std` attribute to exclude the standard library from compilation:

```rust
// main.rs

#![no_std]
```

## Panic Implementation

Rust's standard library provides a default "panic handler" that will be called in response to any panic in our program. By excluding `std` in our project, the compiler will expect us to provide our own panic handler:

```rust
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

Here, [`PanicInfo`](https://doc.rust-lang.org/std/panic/struct.PanicInfo.html) is a `struct`, provided by the `std::panic` module, that holds information about the source of a panic:

```rust
// rustlib\src\rust\library\core\src\panic\panic_info.rs

#[derive(Debug)]
pub struct PanicInfo<'a> {
    payload: &'a (dyn Any + Send),
    message: Option<&'a fmt::Arguments<'a>>,
    location: &'a Location<'a>,
    can_unwind: bool,
    force_no_backtrace: bool,
}
```

We mark our handler function as a _diverging function_—one that never returns to its caller—by having the function endlessly loop. 

## Language Items

Above, `panic_handler` is one of Rust's _language items_: special types and functions required internally by the compiler. Another language item, `eh_personality`, defines behavior for stack-unwinding in the event of a panic. The unwinding procedure typically involves de-allocating stack memory before yielding to a parent thread.

The standard library provides a default implementation that the compiler will look to use. Without access to `std`, we could either (1) provide our own `eh_personality` function, or (2) avoid stack-unwinding altogether.

Rust's [`panic`](https://doc.rust-lang.org/cargo/reference/profiles.html#panic) profile setting lets us specify the [panic strategy](https://doc.rust-lang.org/rustc/codegen-options/index.html#panic) used by our program. Supported values are `"unwind"` and `"abort"`. We can disable stack-unwinding in all our builds with the following Cargo configuration:

```toml
# Cargo.toml

[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```
## Entrypoint

By default, `rustc` will look for a `main()` function to use as the entry-point for our program. `main()` is called by the Rust runtime, and it makes some assumptions about the environment in which it is called; for example, `main()` assumes the existence of some command-line arguments. For this reason, using `main()` as our entrypoint isn't suited for a bare-metal program.

When a Rust program is linked to the Rust standard library, it's execution begins in a C runtime library called `crt0()`; the C runtime, provided by the operating system, sets up an execution environment for a C program, i.e., by creating a stack and placing some arguments into registers. The C runtime then yields to the Rust runtime by calling a designated `start()` function. The minimal Rust runtime performs some brief chores—like setting up stack-overflow guards and handling backtrace prints on panic—before finally calling `main()`.

Because we aren't linking to the standard library, we'll need to provide our own `_start()` function that the C runtime can call into:

```rust
// main.rs

#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}
```

## Linker

Attempting to compile our program on our host (development) system generates a linker error:

```
error: linking with `link.exe` failed: exit code: 1561

LINK : fatal error LNK1561: entry point must be defined
```

This happens because the Rust compiler asserts that a C runtime exists on our host system. We can hint to the linker that no C runtime is available by specifying a bare-metal build target:

```
rustup target add thumbv7em-none-eabihf

cargo build --target thumbv7em-none-eabihf
```

Since the bare-metal target has no C runtime, the linker does not attempt to link to it, and the error disappears.
