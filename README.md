# atomic-pool

[![Documentation](https://docs.rs/atomic-pool/badge.svg)](https://docs.rs/atomic-pool)

Statically allocated pool providing a std-like Box.

## Support for targets without atomic CAS

This crate uses [`portable-atomic`](https://crates.io/crates/portable-atomic) to polyfill atomic
compare-and-swap operations on targets without native hardware support for it.

To use it, you must add a dependency on `portable-atomic` and enable one of its features to
specify how the polyfilling is done. The feature is typically `unsafe-assume-single-core` for
single-core chips running in supervisor mode, or `critical-section` otherwise. Check `portable-atomic`'s
README for more details.

```toml
[dependencies]
atomic-pool = "2.0"
portable-atomic = { version = "1", default-features = false, features = ["critical-section"] }
```

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
