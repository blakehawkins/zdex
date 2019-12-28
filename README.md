# Zdex &emsp; [![Latest version]][crates.io] [![License]][crates.io]

[Latest version]: https://img.shields.io/crates/v/zdex.svg
[crates.io]: https://crates.io/crates/zdex
[License]: https://img.shields.io/crates/l/zdex.svg

Evaluate [Z-order indexing](https://aws.amazon.com/blogs/database/z-order-indexing-for-multifaceted-queries-in-amazon-dynamodb-part-1/) for types, iterators, and tuples of [BitCollection](https://crates.io/crates/bit_collection)s.

## Example

```
use zdex::*;

fn main() -> Result<(), std::io::Error> {
  let v1: FromU8 = 0b0011.into();
  let v2: FromU8 = 0b1111.into();

  // Prints "Vob[01011111]".
  println!("{:?}", (v1, v2).z_index()?);

  Ok(())
}
```
