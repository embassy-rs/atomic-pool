# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased 

- Clippy warning `manually reimplementing div_ceil` when using `pool!()` in nightly in your own application.
- Fixed many clippy suggestions.

## 2.0.0 - 2025-01-02

- Replace deprecated `atomic-polyfill` crate with `portable-atomic`.

## 1.0.1 - 2022-12-11

- Use `atomic-polyfill`, to support targets without atomic CAS.

## 1.0.0 - 2022-08-18

- First release
