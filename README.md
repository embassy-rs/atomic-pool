# atomic-pool

[![Documentation](https://docs.rs/atomic-pool/badge.svg)](https://docs.rs/atomic-pool)

Statically allocated pool providing a std-like Box.

## Optional Features
- `async`<br>
Allow to asynchronously wait for a pool slot to become available. This feature requires the `AtomicWaker` functionality from the `embassy-sync` crate, which in turn requires a critical section implementation like [critical-section](https://crates.io/crates/critical-section).

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
