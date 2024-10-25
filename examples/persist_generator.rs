use std::sync::Arc;

use snowflake_ng::{provider::StdProvider, PersistedSnowflakeGenerator, SnowflakeGenerator};

#[tokio::main]
async fn main() {
    // Use `Arc` wrap it
    let generator = Arc::new(SnowflakeGenerator::default());

    // Our persist generator
    let generator = PersistedSnowflakeGenerator::new(generator, Arc::new(StdProvider));

    // And...multithreading!
    let tasks = (0..100).map(|_| {
        let generator = generator.clone();
        tokio::spawn(async move { generator.assign().await })
    });

    let bucket = futures::future::join_all(tasks)
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let result = bucket
        .into_iter()
        .map(|it| format!("{:b} -> {}", *it, *it))
        .collect::<Vec<_>>()
        .join("\n");

    println!("{result}")
}
