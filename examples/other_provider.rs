// Copyright 2024 Krysztal Huang
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use snowflake_ng::{provider::CHRONO_PROVIDER, SnowflakeGenerator};

fn main() {
    let generator = SnowflakeGenerator::default();

    let mut bucket = Vec::with_capacity(1000);
    for _ in 0..=1000 {
        // Use `CHRONO_PROVIDER` here
        bucket.push(generator.assign_sync(&CHRONO_PROVIDER));
    }

    let result = bucket
        .iter()
        // `Snowflake` can be deref to `i64`
        .map(|it| format!("{:b} -> {}", **it, **it))
        .collect::<Vec<_>>()
        .join("\n");

    println!("{result}")
}
