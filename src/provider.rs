// Copyright 2024 Krysztal Huang
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::time::UNIX_EPOCH;

use crate::TimeProvider;

pub static STD_PROVIDER: StdProvider = StdProvider;

#[cfg(feature = "chrono")]
pub static CHRONO_PROVIDER: ChronoProvider = ChronoProvider;

#[cfg(feature = "time")]
pub static TIME_CRATE_PROVIDER: TimeCrateProvider = TimeCrateProvider;

#[derive(Debug)]
pub struct StdProvider;

impl TimeProvider for StdProvider {
    fn timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}
unsafe impl Sync for StdProvider {}
unsafe impl Send for StdProvider {}

#[cfg(feature = "chrono")]
#[derive(Debug)]
pub struct ChronoProvider;

#[cfg(feature = "chrono")]
impl TimeProvider for ChronoProvider {
    fn timestamp(&self) -> u64 {
        chrono::Local::now().timestamp_millis() as u64
    }
}

#[cfg(feature = "chrono")]
unsafe impl Sync for ChronoProvider {}
#[cfg(feature = "chrono")]
unsafe impl Send for ChronoProvider {}

#[cfg(feature = "time")]
#[derive(Debug)]
pub struct TimeCrateProvider;

#[cfg(feature = "time")]
impl TimeProvider for TimeCrateProvider {
    fn timestamp(&self) -> u64 {
        (time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as u64
    }
}

#[cfg(feature = "time")]
unsafe impl Sync for TimeCrateProvider {}
#[cfg(feature = "time")]
unsafe impl Send for TimeCrateProvider {}
