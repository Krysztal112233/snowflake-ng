# snowflake-ng

Dead easy and high performance `snowflake` implemented in Rust.

This crate only implemented Twitter(formally X)'s snowflake.

## Why `ng` or `next-generation`?

Actually, this crate doesn't **_next-generation_** enough.

The use of `-ng` is simply to distinguish between the two existing implementations:

- [snowflake-rs](https://github.com/BinChengZhao/snowflake-rs)
- [snowflake](https://crates.io/crates/snowflake)

Maybe lock-free can be a feature of `next-generation` :)

## Whats the different between them?

- The `snowflake` crate is completely unmaintained and it's even using Rust 2018 Edition.
- The `snowflake-rs` (`rs-snowflake` at crates.io) doesn't support async/await.

## Why we need async?

What we need to know is that only 4096 `Snowflake ID` can be generated in a millisecond in the standard implementation.

If we assigned out the sequence(from 0 to 4095), we have to waiting for one millisecond. If we doesn't use the asynchronous, we have to **_sleep for one millisecond_**!

But at the same time, this crate also provided synchronous function, but you have to enable `sync` feature:

```toml
snowflake-ng = { version = "0.1", features = ["sync"]}
```

## Thread safety?

YES!

Inner data use `Atomic*` type to keep lock-free and atomic update.

So you can share `SnowflakeGenerator` between threads safety, or make a global static one!

## How to use?

Firstly, add this crate to your `cargo.toml`:

```toml
snowflake-ng = "0.1"
```

This crate provide some extra `TimeProvider` implementation based different crate:

- `time`
- `chrono`

And provide basic implementation based standard library: `std::time::SystemTime`

If you want to accelerate your build time, you can disable all the features to avoid introduce extra build dependenciesD.

After add to `cargo.toml`, you can made your own `SnowflakeGenerator`:

```rust
let generator = SnowflakeGenerator::default();
```

Then, start your generation:

```rust
generator.assign_sync(&STD_PROVIDER)
```

Please see [example](./examples/),for more example such as async support and custom identifier.
