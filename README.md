# Zdex &emsp; [![Latest version]][crates.io] [![License]][crates.io]

[Latest version]: https://img.shields.io/crates/v/zdex.svg
[crates.io]: https://crates.io/crates/zdex
[License]: https://img.shields.io/crates/l/zdex.svg

Evaluate [Z-order indexing](https://aws.amazon.com/blogs/database/z-order-indexing-for-multifaceted-queries-in-amazon-dynamodb-part-1/) for types, iterators, and tuples of [BitCollection](https://crates.io/crates/bit_collection)s.

See also [`morton_encoding`](https://crates.io/crates/morton_encoding).

## Examples

Basic example using built-in `FromU8` BitCollection:

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

Example with custom BitCollection and high dimensionality:

```rust
use bit_collection::*;
use zdex::*;

#[bit(BitU8, mask = "!0", retr = "0")]
#[derive(BitCollection, Clone, Copy, Debug)]
pub struct BigVal(u128);

fn main() -> Result<(), std::io::Error> {
    let vals: Vec<BigVal> = [BigVal(1u128 << 90)]
        .into_iter()
        .cycle()
        .take(10)
        .map(|r| r.to_owned())
        .collect();

    let z_index = vals.z_index()?;

    // Prints out a large vob, followed by the numbers 10 and 900.
    println!(
        "{:?}\n  {}\n  {}",
        z_index,
        z_index.iter_set_bits(..).count(),
        z_index.iter_unset_bits(..).count()
    );

    Ok(())
}
```

## Todo

- [x] docs example: custom BitCollections
- [ ] docs example: practical example with z-order index ranges
- [ ] docs example: manipulating result vob
- [ ] docs quality: rustdoc + docs.rs link
- [ ] key feature: z-indexes over heterogeneous `BitCollections`
- [ ] key feature: `is_relevant` and `next_jump_in`
- [ ] feature: iterator over sub-ranges (Page Jump Querying heuristic)
- [ ] docs metadata: crates.io tags
