<div align="center">

# Inception

A Rust macro for writing Rust macros.

</div>

---

Inception is a replacement for `macro_rules!`. It aims to explore the design space of declarative macros, experiment with different syntax, and enable proposals for improving or superseding `macro_rules!`.

Inception is currently _not_ intended to be used productively, since it is incomplete and might change drastically in the future. Since inception is intended for experimentation, expect frequent breaking changes.

## Status

- [x] working prototype
- [ ] can parse all kinds of fragments supported by `macro_rules!`
- [ ] can parse all kinds of Rust syntax in a versioned manner
- [ ] thorough documentation
- [ ] published on crates.io

## License

Licensed under either of [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT license](https://opensource.org/license/mit) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
