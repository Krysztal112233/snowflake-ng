// Copyright 2024 Krysztal Huang
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![doc = include_str!("../README.md")]

use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use futures::executor;
use futures_timer::Delay;
use rand::RngCore;

pub mod provider;

pub trait TimeProvider {
    /// Timestamp fetcher.
    fn timestamp(&self) -> u64;
}

/// Generated [`Snowflake`](Snowflake)
///
/// # Implementation
///
/// Let me describe snowflake ID (*SID in below*) in simple words.
///
/// Firstly, we have to know this structure of SID.
///
/// SIDs are actually [i64] types. It's length 64bit and 1bit for sign.
///
/// So it looks like this:
///
/// ```text
/// | sign |   data    | # sign not used.
/// | 1bit |   63bit   |
/// ```
///
/// Next, I'll introduce standard SID design to you. Why STANDARD? Because there are some variant, just ignore them use Twitter's(formally X) design only.
///
/// The standard SID contains these content:
///
/// - Timestamp: 41bit
/// - Identifier(or Machine ID?): 10bit
/// - Sequence Number: 12bit
///
/// Our SID structure looks like this
/// ```text
/// | sign |                data                      |
/// |   0  | Timestamp | Identifier | Sequence Number |
/// | 1bit |   41bit   |    10bit   |     12bit       |
/// ```
///
/// ✨ So cool, you in just understood the SID structure!
///
/// Ok, let's deep in **_DARK_**.
///
/// ## Timestamp
///
/// In standard design, timestamp can start at any time.
///
/// But here, the precision we need for the timestamp is to the millisecond, so exactly 41bits.
///
/// ## Identifier
///
/// Base the design of distributed systems, we will have many machine(or instance) running at same time.
///
/// So we must distinguish between them. Based identifier have 10bit, we can have 1024 instance at same time, thats so cool!
///
/// ## Sequence Number
///
/// Have you just noticed the `Sequence Number`? It have 12bit, means it can process at most 4096 message(or other things if you want) in one millisecond.
///
/// Above all, we can know: the entire system can produce at most `1024 * 4096 = 4194304` pieces of message at one millisecond!
///
/// ## Out of assigned
///
/// But there is always the possibility that we will encounter a situation: all the SIDs for this millisecond have been assigned!
///
/// At this time, the instance must waiting for next millisecond. At next millisecond, we will have new 4096 SID can be assigned.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Snowflake(i64);

impl From<Snowflake> for i64 {
    fn from(value: Snowflake) -> Self {
        value.0
    }
}

impl Deref for Snowflake {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<i64> for Snowflake {
    fn as_ref(&self) -> &i64 {
        self
    }
}

#[derive(Debug)]
pub struct SnowflakeConfiguration {
    /// Identifier ID
    ///
    /// [`SnowflakeGenerator`](SnowflakeGenerator) will use **_10bit_**
    ///
    /// By default, `identifier_id` set to the number generated by `rand` crate.
    pub identifier: u64,
}

impl SnowflakeConfiguration {
    pub fn with_identifier(identifier: u64) -> Self {
        Self { identifier }
    }
}

impl Default for SnowflakeConfiguration {
    fn default() -> Self {
        Self {
            identifier: rand::thread_rng().next_u64(),
        }
    }
}

unsafe impl Send for SnowflakeConfiguration {}

/// Filling timestamp by mask  
fn fill_timestamp(sid: u64, timestamp: u64) -> u64 {
    const MASK: u64 = (1u64 << 41) - 1;
    let truncated_timestamp = timestamp & MASK; // Make sure `timestamp` up to 41bit
    let filled = truncated_timestamp << 22;
    (sid & !(MASK << 22)) | filled
}

/// Filling identifier by mask
fn fill_identifier(sid: u64, identifier: u64) -> u64 {
    const MASK: u64 = (1u64 << 10) - 1; // 限定为10位
    let truncated_identifier = identifier & MASK; // Make sure `identifier` up to 10bit
    let filled = truncated_identifier << 12;
    (sid & !(MASK << 12)) | filled
}

/// Filling sequence by mask
fn fill_sequence(sid: u64, sequence: u64) -> u64 {
    const MASK: u64 = (1u64 << 12) - 1;
    let truncated_sequence = sequence & MASK; // // Make sure `sequence` up to 12bit

    // Does not need to shift
    (sid & !MASK) | truncated_sequence
}

pub fn filling<T0, T1, T2>(dest: u64, timestamp: T0, identifier: T1, sequence: T2) -> u64
where
    T0: Into<u64>,
    T1: Into<u64>,
    T2: Into<u64>,
{
    let sid = fill_timestamp(dest, timestamp.into());
    let sid = fill_identifier(sid, identifier.into());
    fill_sequence(sid, sequence.into())
}

/// Generating [`Snowflake`](Snowflake)
///
/// Recommended keep this generator single-instance for one instance's SID generation.
///
/// # Thread safety
///
/// You can use [`::std::sync::Arc`](::std::sync::Arc) sharing ownership between thread.
#[derive(Debug, Default)]
pub struct SnowflakeGenerator {
    timestamp_sequence: AtomicU64,
    cfg: SnowflakeConfiguration,
}
const MAX_SEQUENCE: u16 = 0xFFF; // 12bit sequence

impl SnowflakeGenerator {
    pub fn with_cfg(cfg: SnowflakeConfiguration) -> Self {
        Self {
            cfg,
            timestamp_sequence: AtomicU64::new(0),
        }
    }

    /// Assign a [`Snowflake`](Snowflake) with [`TimeProvider`](TimeProvider)
    pub async fn assign<T>(&self, provider: &T) -> Snowflake
    where
        T: TimeProvider + Sync + Send,
    {
        loop {
            let timestamp = provider.timestamp();
            let current = self.timestamp_sequence.load(Ordering::Relaxed);
            let current_timestamp = current >> 16;
            let current_sequence = (current & 0xFFFF) as u16;

            match current_timestamp.cmp(&timestamp) {
                std::cmp::Ordering::Less => {
                    // update timestamp
                    let new_value = timestamp << 16;

                    if self
                        .timestamp_sequence
                        .compare_exchange(current, new_value, Ordering::SeqCst, Ordering::SeqCst)
                        .is_ok()
                    {
                        let sid = fill_timestamp(0, timestamp);
                        let sid = fill_identifier(sid, self.cfg.identifier);
                        let sid = fill_sequence(sid, 0);
                        return Snowflake(sid as i64);
                    }
                }
                std::cmp::Ordering::Equal => {
                    if current_sequence >= MAX_SEQUENCE {
                        // Sequence reached MAX, waiting for next millisecond
                        Delay::new(Duration::from_millis(1)).await;
                        continue;
                    }

                    let new_sequence = current_sequence + 1;
                    let new_value = (timestamp << 16) | new_sequence as u64;

                    if self
                        .timestamp_sequence
                        .compare_exchange(current, new_value, Ordering::SeqCst, Ordering::SeqCst)
                        .is_ok()
                    {
                        let sid = fill_timestamp(0, timestamp);
                        let sid = fill_identifier(sid, self.cfg.identifier);
                        let sid = fill_sequence(sid, new_sequence as u64);
                        return Snowflake(sid as i64);
                    }
                }
                std::cmp::Ordering::Greater => Delay::new(Duration::from_millis(1)).await,
            };
        }
    }

    /// Assign a new [`Snowflake`](Snowflake) but in synchronous way.
    #[cfg(feature = "sync")]
    pub fn assign_sync<T>(&self, provider: &T) -> Snowflake
    where
        T: TimeProvider + Sync + Send,
    {
        executor::block_on(self.assign(provider))
    }
}

/// Persisted [`SnowflakeGenerator`](SnowflakeGenerator).
///
/// Designed for easier contextualization.
///
/// # Thread safety
///
/// YES. You can use [`::std::sync::Arc`](::std::sync::Arc) to send data between threads safety.
///
/// # Clone
///
/// Clone is cheap. If you clone it, it equals invoke [`Arc::clone`](Arc::clone) three times.
#[derive(Debug)]
pub struct PersistedSnowflakeGenerator<T> {
    generator: Arc<SnowflakeGenerator>,
    provider: Arc<T>,
}

impl<T> PersistedSnowflakeGenerator<T>
where
    T: TimeProvider + Send + Sync,
{
    /// Constructing new [`PersistedSnowflakeGenerator`](PersistedSnowflakeGenerator) from already instanced [`SnowflakeGenerator`](SnowflakeGenerator) and [`TimeProvider`](TimeProvider)
    ///
    /// The cost very low, please relax to constructing your [`PersistedSnowflakeGenerator`](PersistedSnowflakeGenerator).
    ///
    /// # Thread safety
    ///
    /// Yes, `time_provider` must be send and sync between threads and [`SnowflakeGenerator`](SnowflakeGenerator) are already thread safe.
    pub fn new(generator: Arc<SnowflakeGenerator>, provider: Arc<T>) -> Self {
        Self {
            generator,
            provider,
        }
    }

    /// Assign a new [`Snowflake`](Snowflake)
    pub async fn assign(&self) -> Snowflake {
        self.generator.assign(self.provider.as_ref()).await
    }

    /// Assign a new [`Snowflake`](Snowflake) but in synchronous way.
    #[cfg(feature = "sync")]
    pub fn assign_sync(&self) -> Snowflake {
        self.generator.assign_sync(self.provider.as_ref())
    }
}

impl<T> Clone for PersistedSnowflakeGenerator<T> {
    fn clone(&self) -> Self {
        Self {
            generator: self.generator.clone(),
            provider: self.provider.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, sync::Arc};

    use parking_lot::RwLock;
    use provider::{StdProvider, STD_PROVIDER};

    use super::*;

    #[test]
    fn test_fill_timestamp() {
        // Case1
        let sid = 0u64;
        let timestamp = 0b101010;
        let expected = 42 << 22;
        let result = fill_timestamp(sid, timestamp);
        assert_eq!(result, expected);

        // Case2
        let sid = 0u64;
        let timestamp = (1u64 << 42) - 1;
        let expected = ((1u64 << 41) - 1) << 22;
        let result = fill_timestamp(sid, timestamp);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_fill_identifier() {
        // Case1
        let sid = 0u64;
        let identifier = 0b110101;
        let expected = 53 << 12;
        let result = fill_identifier(sid, identifier);
        assert_eq!(result, expected);

        // Case2
        let sid = 0u64;
        let identifier = (1u64 << 11) - 1;
        let expected = ((1u64 << 10) - 1) << 12;
        let result = fill_identifier(sid, identifier);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_fill_sequence() {
        // Case1
        let sid = 0u64;
        let sequence = 0b1001;
        let expected = 9;
        let result = fill_sequence(sid, sequence);
        assert_eq!(result, expected);

        // Case2
        let sid = 0u64;
        let sequence = (1u64 << 13) - 1;
        let expected = (1u64 << 12) - 1;
        let result = fill_sequence(sid, sequence);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_filling() {
        let sid = 0u64;
        let timestamp = 0b10101010101010101010101010101010101010101u64;
        let identifier = 0b110101u64;
        let sequence = 0b1001u64;

        let expected = (timestamp << 22) | (identifier << 12) | sequence;

        let result = filling(sid, timestamp, identifier, sequence);
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_assign() {
        let generator = Arc::new(SnowflakeGenerator::default());

        for _ in 0..1024 {
            generator.assign(&provider::TIME_CRATE_PROVIDER).await;
        }
    }

    #[tokio::test]
    async fn test_assign_multithread() {
        let generator = Arc::new(SnowflakeGenerator::default());

        let mut handles = vec![];
        let id_set = Arc::new(RwLock::new(HashSet::new()));

        for _ in 0..1000 {
            let generator = Arc::clone(&generator);
            let id_set = Arc::clone(&id_set);
            let handle = tokio::spawn(async move {
                for _ in 0..1000 {
                    let id = generator.assign(&STD_PROVIDER).await;
                    let mut set = id_set.write();
                    if set.contains(&id) {
                        panic!("Duplicate `Snowflake` generated!");
                    }
                    set.insert(id);
                }
            });
            handles.push(handle);
        }

        futures::future::join_all(handles).await;

        assert_eq!(
            id_set.read().len(),
            1000 * 1000,
            "Some `Snowflake` were lost!"
        );
    }

    #[test]
    fn test_persists() {
        let binding = Arc::new(SnowflakeGenerator::default());
        let persist = PersistedSnowflakeGenerator::new(binding.clone(), Arc::new(StdProvider));

        let snowflakes = (0..1000)
            .map(|_| persist.assign_sync())
            .collect::<HashSet<_>>();

        assert_eq!(snowflakes.len(), 1000);
    }

    #[tokio::test]
    async fn test_persists_multithread() {
        let binding = Arc::new(SnowflakeGenerator::default());

        let persist = Arc::new(PersistedSnowflakeGenerator::new(
            binding.clone(),
            Arc::new(StdProvider),
        ));

        let tasks = (0..1000).map(|_| {
            let persist = persist.clone();
            tokio::spawn(async move { persist.assign().await })
        });
        let snowflakes = futures::future::join_all(tasks).await;

        assert_eq!(snowflakes.len(), 1000);
    }
}
