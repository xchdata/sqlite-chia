# sqlite-chia

An extension for SQLite, written in Rust, which provides Chia utility
functions.

## Installation

Run `cargo build --release --features build_extension` to build a shared
library loadable as SQLite extension.

Optionally, you can manually strip the library to decrease binary size:
`strip --strip-all target/release/libchia.so`.

## Exposed extension functions

- `bech32m_encode(text, blob) -> text`: Takes a prefix as first argument and
  a blob as second and bech32m-encodes the blob with the given prefix into
  a string.
- `bech32m_decode(text) -> blob`: Decodes a bech32m-encoded string into a blob.
- `blob_from_hex(string) -> blob`: Hex-decodes a string ('cafe') into a blob
  (x'cafe').
- `chia_amount_int(blob) -> integer`: Parse a Chia amount blob into an integer
  (representing mojos).
- `chia_fullblock_json(blob) -> text`: Parse a blob holding a Chia-serialized
  block into JSON. The returned text is valid JSON and can be further processed
  using SQLite's JSON functions.

## Dependencies & References

Binding to SQLite's [loadable extension interface][loadext] is handled by
[rusqlite] extended with support for creating loadable extensions (see [pull
request #910][pr910] or [corresponding branch][rusqlite-le]).

[loadext]: https://www.sqlite.org/loadext.html
[pr910]: https://github.com/rusqlite/rusqlite/pull/910
[rusqlite-le]: https://github.com/Genomicsplc/rusqlite/tree/loadable-extensions
[rusqlite]: https://github.com/rusqlite/rusqlite

## License

SPDX-License-Identifier: MIT

Copyright (C) 2022-2023 xchdata.io

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
