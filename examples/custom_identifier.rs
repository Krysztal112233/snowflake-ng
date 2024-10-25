// Copyright 2024 Krysztal Huang
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use snowflake_ng::{provider::STD_PROVIDER, SnowflakeConfiguration, SnowflakeGenerator};

fn main() {
    // Let's get a `SnowflakeGenerator` with `SnowflakeConfiguration`.
    let generator = SnowflakeGenerator::with_cfg(SnowflakeConfiguration::with_identifier(11));

    let mut bucket = Vec::with_capacity(10);
    for _ in 0..=10 {
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
