# uhttp\_content\_encoding -- Parser for HTTP Content-Encoding header

[Documentation](https://docs.rs/uhttp_content_encoding)

This crate provides a zero-allocation, iterator/slice-based parser for extracting HTTP
[content encoding](https://tools.ietf.org/html/rfc7231#section-3.1.2) types as they
appear in the `Content-Encoding` request header. [Standard
encodings](http://www.iana.org/assignments/http-parameters/http-parameters.xhtml#content-coding)
are extracted as enum values, and unknown encodings are extracted as slices
for further processing.

## Example

```rust
use uhttp_content_encoding::{content_encodings, ContentEncoding, StdContentEncoding};

let mut encs = content_encodings(" gzip, identity, custom-enc");
assert_eq!(encs.next(), Some(ContentEncoding::Other("custom-enc")));
assert_eq!(encs.next(), Some(ContentEncoding::Std(StdContentEncoding::Identity)));
assert_eq!(encs.next(), Some(ContentEncoding::Std(StdContentEncoding::Gzip)));
assert_eq!(encs.next(), None);
```

## Usage

This [crate](https://crates.io/crates/uhttp_content_encoding) can be used through cargo by
adding it as a dependency in `Cargo.toml`:

```toml
[dependencies]
uhttp_content_encoding = "0.5.1"
```
and importing it in the crate root:

```rust
extern crate uhttp_content_encoding;
```
