// Copyright 2024 Krysztal Huang
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use snowflake_ng::{provider::STD_PROVIDER, SnowflakeGenerator};

#[tokio::main]
async fn main() {
    // Ok, let's get a `SnowflakeGenerator` with all things defaulted.
    let generator = SnowflakeGenerator::default();

    // And how about get 1000 pieces of `Snowflake`s? I love snowflake. :)
    let mut tasks = Vec::with_capacity(1000);
    for _ in 0..=1000 {
        // We spawn 1000 task to generate `Snowflake`
        tasks.push(generator.assign(&STD_PROVIDER));
    }

    // We get a bucket of `Snowflake`s!
    let bucket = futures::future::join_all(tasks).await;

    let result = bucket
        .iter()
        // `Snowflake` can be deref to `i64`
        .map(|it| format!("{:b} -> {}", **it, **it))
        .collect::<Vec<_>>()
        .join("\n");

    println!("{result}")
}
