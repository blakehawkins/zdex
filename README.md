# Zdex &emsp; [![Latest version]][crates.io] [![License]][crates.io] [![Docs badge]][docs.rs]

[Latest version]: https://img.shields.io/crates/v/zdex.svg
[crates.io]: https://crates.io/crates/zdex
[License]: https://img.shields.io/crates/l/zdex.svg
[Docs badge]: https://img.shields.io/badge/docs.rs-rustdoc-green
[docs.rs]: https://docs.rs/zdex/

Evaluate [Z-order indexing](https://aws.amazon.com/blogs/database/z-order-indexing-for-multifaceted-queries-in-amazon-dynamodb-part-1/) for types, iterators, and tuples of [BitCollection](https://crates.io/crates/bit_collection)s.

See also [`morton_encoding`](https://crates.io/crates/morton_encoding).

## Examples

### Basic example using built-in `FromU8` BitCollection:

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

### Example with custom BitCollection and high dimensionality:

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

### Practical example: querying over a (lon, lat) z-index:

```rust
use bit_collection::*;
use std::collections::BTreeMap;
use zdex::*;

#[bit(BitU8, mask = "!0", retr = "0")]
#[derive(BitCollection, Clone, Copy, Debug)]
pub struct LonLatVal(i32);

fn lonlat_val(lonlat: (f32, f32)) -> (LonLatVal, LonLatVal) {
    (LonLatVal(lonlat.0 as i32), LonLatVal(lonlat.1 as i32))
}

fn main() -> Result<(), std::io::Error> {
    let database = vec![
        ((  -0.127758,  51.507351), "London"),
        ((   2.352222,  48.856613), "Paris"),
        ((  10.752245,  59.913868), "Oslo"),
        (( -74.005974,  40.712776), "New York"),
        ((-118.243683,  34.052235), "Los Angeles"),
        (( -46.633308, -23.550520), "Sao Paolo"),
        (( 151.209290, -33.868820), "Sydney"),
    ].into_iter()
    .map(|(lonlat, loc)| (
        lonlat_val(lonlat),
        loc
    )).map(|(lonlat, loc)| (
        lonlat
            .z_index()
            .map(|v| v.iter_storage().next().expect("empty vob"))
            .expect("failed to produce z-index"),
            loc
    )).collect::<BTreeMap<_, _>>();

    let usa_query = (
        lonlat_val((-200.0, 20.0))
            .z_index()?
            .iter_storage()
            .next()
            .expect("empty vob")
    )..(
        lonlat_val((-50.0, 50.0))
            .z_index()?
            .iter_storage()
            .next()
            .expect("empty vob")
    );

    // Prints `["New York", "Los Angeles"]`.
    println!(
        "{:?}",
        database.range(usa_query).map(|(_k, v)| v).collect::<Vec<_>>()
    );

    Ok(())
}
```

## Todo

- [x] docs example: custom BitCollections
- [x] docs example: practical example with z-order index ranges
- [x] docs example: manipulating result vob
- [x] docs quality: rustdoc + docs.rs link
- [ ] key feature: z-indexes over heterogeneous `BitCollections`
- [ ] key feature: `is_relevant` and `next_jump_in`
- [ ] feature: iterator over sub-ranges (Page Jump Querying heuristic)
- [ ] docs metadata: crates.io tags
- [ ] code quality: rustfmt + clippy
