# toml-spanned-value

[![dependency status](https://deps.rs/repo/github/est31/toml-spanned-value/status.svg)](https://deps.rs/repo/github/est31/toml-spanned-value)

The [toml](https://github.com/alexcrichton/toml-rs) crate provides users with a `Value` type that can be used for custom parsing logic. However, this type doesn't support span information. Spans provide location information to allow for good error messages or other purposes. The `toml-spanned-value` crate provides a `SpannedValue` type to fix this.

See also these issues/threads in `toml-rs`:

* https://github.com/alexcrichton/toml-rs/issues/95
* https://github.com/alexcrichton/toml-rs/pull/327

### License
[license]: #license

This library is distributed under the terms of both the MIT license
and the Apache License (Version 2.0), at your option.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

#### License of your contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
