# holodeque

Array- and slice-backed double-ended queues in 100% safe Rust.

---

This crate provides `ArrayDeque` and `SliceDeque`, fixed-size ring buffers with
interfaces similar to the standard library's `VecDeque`.

`holodeque` makes use of the unstable `array_map` feature to provide `Default`
initialization of arbitrarily-sized arrays. As a result, **a `nightly` compiler
is required until this feature is stabilized**. See the [tracking issue] for its
current status.

[tracking issue]: https://github.com/rust-lang/rust/issues/75243

## License

Licensed under either of

- Apache License, Version 2.0 (`LICENSE-APACHE` or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (`LICENSE-MIT` or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
