// Copyright 2024 Krysztal Huang
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use snowflake_ng::{provider::STD_PROVIDER, SnowflakeGenerator};

fn main() {
    // Ok, let's get a `SnowflakeGenerator` with all things defaulted.
    let generator = SnowflakeGenerator::default();

    // And how about get 1000 pieces of `Snowflake`s? I love snowflake. :)
    let mut bucket = Vec::with_capacity(1000);
    for _ in 0..=1000 {
        // We can use `STD_PROVIDER` for better performance, or other in the `provider` crate.
        // `TimeProvider` are used to generate timestamp (u64)
        bucket.push(generator.assign_sync(&STD_PROVIDER));
    }

    let result = bucket
        .iter()
        // `Snowflake` can be deref to `i64`
        .map(|it| format!("{:b} -> {}", **it, **it))
        .collect::<Vec<_>>()
        .join("\n");

    println!("{result}")
}
