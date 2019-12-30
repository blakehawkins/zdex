# Zdex &emsp; [![Latest version]][crates.io] [![License]][crates.io] [![Docs badge]][docs.rs] [![Issues badge]][issues]

[Latest version]: https://img.shields.io/crates/v/zdex.svg
[crates.io]: https://crates.io/crates/zdex
[License]: https://img.shields.io/crates/l/zdex.svg
[Docs badge]: https://img.shields.io/badge/docs.rs-rustdoc-green
[docs.rs]: https://docs.rs/zdex/
[Issues badge]: https://img.shields.io/github/issues/blakehawkins/zdex
[issues]: https://github.com/blakehawkins/zdex/issues

Evaluate [Z-order indexing](https://aws.amazon.com/blogs/database/z-order-indexing-for-multifaceted-queries-in-amazon-dynamodb-part-1/) for types, iterators, and tuples of [BitCollection](https://crates.io/crates/bit_collection)s.

See also [`morton_encoding`](https://crates.io/crates/morton_encoding).

## Example

Here's a basic example using the built-in `FromU8` BitCollection - see more
examples in the [docs](https://docs.rs/zdex).

```rust
use zdex::*;

fn main() -> Result<(), std::io::Error> {
  let v1: FromU8 = 0b0011.into();
  let v2: FromU8 = 0b1111.into();

  // Prints "Vob[01011111]".
  println!("{:?}", (v1, v2).z_index()?);

  Ok(())
}
```
