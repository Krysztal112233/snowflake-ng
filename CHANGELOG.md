# Changelog

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.3

### Add

- Add `SnowflakeGenerator` for basic generation
- Add `Snowflake` as generated result
- Add `TimeProvider` for custom timestamp and epoch
- Add `StdProvider` for none extra dependencies required implementation
- Add `ChronoProvider` for more custom timestamp implementation
- Add `TimeCrateProvider` for fast(than _chrono_ but slow than _std_) and some custom timestamp implementation
- Add feature `sync` to enable sync `Snowflake` assign

## v0.1.4

### Add

- Add `PersistedSnowflakeGenerator` for some convenience

## v0.1.5

### Add

- Add `SnowflakeId` as alias for `i64`

### Changes

- Make documents more beautiful

## Unreleased
