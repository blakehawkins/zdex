//! Zdex is for evaluating z-order indexes (morton encoding) for types,
//! iterators, and tuples of
//! [BitCollections](https://crates.io/crates/bit_collection).
//!
//! Z-order indexing is a database range-querying technique to optimise
//! scanning performance, and Zdex aims to be prescriptive in providing that
//! functionality.
//!
//! ## Examples
//!
//! ### Basic example using built-in `FromU8` BitCollection:
//!
//! ```rust
//! use zdex::*;
//!
//! fn main() -> Result<(), std::io::Error> {
//!   let v1: FromU8 = 0b0011.into();
//!   let v2: FromU8 = 0b1111.into();
//!
//!   // Prints "Vob[01011111]".
//!   println!("{:?}", (v1, v2).z_index()?);
//!
//!   Ok(())
//! }
//! ```
//!
//! ### Example with custom BitCollection and high dimensionality:
//!
//! ```rust
//! use bit_collection::*;
//! use zdex::*;
//!
//! #[bit(BitU8, mask = "!0", retr = "0")]
//! #[derive(BitCollection, Clone, Copy, Debug)]
//! pub struct BigVal(u128);
//!
//! fn main() -> Result<(), std::io::Error> {
//!     let vals: Vec<BigVal> = [BigVal(1u128 << 90)]
//!         .into_iter()
//!         .cycle()
//!         .take(10)
//!         .map(|r| r.to_owned())
//!         .collect();
//!
//!     let z_index = vals.z_index()?;
//!
//!     // Prints out a large vob, followed by the numbers 10 and 900.
//!     println!(
//!         "{:?}\n  {}\n  {}",
//!         z_index,
//!         z_index.iter_set_bits(..).count(),
//!         z_index.iter_unset_bits(..).count()
//!     );
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Practical example: querying over a (lon, lat) z-index:
//!
//! ```rust
//! use bit_collection::*;
//! use std::collections::BTreeMap;
//! use zdex::*;
//!
//! #[bit(BitU8, mask = "!0", retr = "0")]
//! #[derive(BitCollection, Clone, Copy, Debug)]
//! pub struct LonLatVal(i32);
//!
//! fn lonlat_val(lonlat: (f32, f32)) -> (LonLatVal, LonLatVal) {
//!     (LonLatVal(lonlat.0 as i32), LonLatVal(lonlat.1 as i32))
//! }
//!
//! fn main() -> Result<(), std::io::Error> {
//!     let database = vec![
//!         ((  -0.127758,  51.507351), "London"),
//!         ((   2.352222,  48.856613), "Paris"),
//!         ((  10.752245,  59.913868), "Oslo"),
//!         (( -74.005974,  40.712776), "New York"),
//!         ((-118.243683,  34.052235), "Los Angeles"),
//!         (( -46.633308, -23.550520), "Sao Paolo"),
//!         (( 151.209290, -33.868820), "Sydney"),
//!     ].into_iter()
//!     .map(|(lonlat, loc)| (
//!         lonlat_val(lonlat),
//!         loc
//!     )).map(|(lonlat, loc)| (
//!         lonlat
//!             .z_index()
//!             .map(|v| v.iter_storage().next().expect("empty vob"))
//!             .expect("failed to produce z-index"),
//!             loc
//!     )).collect::<BTreeMap<_, _>>();
//!
//!     let usa_query = (
//!         lonlat_val((-200.0, 20.0))
//!             .z_index()?
//!             .iter_storage()
//!             .next()
//!             .expect("empty vob")
//!     )..(
//!         lonlat_val((-50.0, 50.0))
//!             .z_index()?
//!             .iter_storage()
//!             .next()
//!             .expect("empty vob")
//!     );
//!
//!     // Prints `["New York", "Los Angeles"]`.
//!     println!(
//!         "{:?}",
//!         database.range(usa_query).map(|(_k, v)| v).collect::<Vec<_>>()
//!     );
//!
//!     Ok(())
//! }
//! ```


use bit_collection::{BitIter, BitCollection};
use vob::{vob, Vob};


/// Trait for implementing `z_index()` for `BitCollection`s.  A blanket
/// implementation is provided for `BitCollection<Item=BitU8>`.
pub trait Zdexed {
    fn z_index(self) -> std::io::Result<vob::Vob>;
}

/// A trait for implementing `Zdexed` over iterables.  A blanket implementation
/// is provided for `IntoIter<T: Zdexed>`.
pub trait ZdexedIter {
    fn z_index(self) -> std::io::Result<vob::Vob>;
}

/// A trait for implementing `Zdexed` over tuples.  A blanket implementation is
/// provided for homogeneous 2-, 3-, and 4- tuples.
pub trait ZdexedTup {
    fn z_index(self) -> std::io::Result<vob::Vob>;
}

impl<T> Zdexed for T
    where T: BitCollection<Item=BitU8> + Copy + std::fmt::Debug
{
    fn z_index(self) -> std::io::Result<vob::Vob> {
        let size: usize = self
            .clone()
            .as_iter()
            .map(|v| v.0 as usize)
            .max()
            .unwrap_or(0);
        
        let mut vob_init = vob![];
        vob_init.resize(size + 1, false);

        Ok(self
           .clone()
           .as_iter()
           .fold(vob_init, |mut vob, idx| {
               vob.set(vob.len() - (idx.0 + 1) as usize, true);

               vob
           })
        )
    }
}

impl<T, U> ZdexedIter for T
    where T: IntoIterator<Item=U>,
          U: Zdexed
{
    fn z_index(self) -> std::io::Result<vob::Vob> {
        let vobs: Vec<vob::Vob> = self
            .into_iter()
            .map(|z| z.z_index())
            .collect::<Result<Vec<_>, _>>()?;

        let size = vobs.iter().map(|v| v.len()).max().unwrap_or(0);

        let vobs: Vec<vob::Vob> = vobs
            .into_iter()
            .map(|mut v| {
                let diff = size - v.len();
                v.resize(size, false);

                let mut v = v.into_iter().collect::<Vec<_>>();
                
                for _ in 0..diff {
                    v.rotate_right(1);
                }
                
                v.into_iter().collect()
            })
            .collect();

        let mut res = vob![];
        res.resize(vobs.len() * (size), false);

        let mut vobs = vobs
            .iter()
            .map(|ref v| v.iter().peekable())
            .collect::<Vec<std::iter::Peekable<vob::Iter<usize>>>>();

        let mut i = 0;
        loop {
            if vobs[0].peek().is_none() {
                break;
            }

            res.set(i, vobs[0].next().expect("peeked"));
            i += 1;
            vobs.rotate_left(1);
        }

        Ok(res)
    }
}

impl ZdexedTup for (Zdexed, Zdexed)
{
    fn z_index(self) -> std::io::Result<vob::Vob> {
        vec![self.0, self.1].into_iter().z_index()
    }
}

impl<T> ZdexedTup for (T, T, T)
    where T: Zdexed
{
    fn z_index(self) -> std::io::Result<vob::Vob> {
        vec![self.0, self.1, self.2].into_iter().z_index()
    }
}

impl<T> ZdexedTup for (T, T, T, T)
    where T: Zdexed
{
    fn z_index(self) -> std::io::Result<vob::Vob> {
        vec![self.0, self.1, self.2, self.3].into_iter().z_index()
    }
}

/// The `BitCollection::Item`-type prescribed by Zdex for use in
/// `Zdexed`-compatible `BitCollections`.  Custom `BitCollection`s specified
/// for use with Zdex must therefore specify `#[bit(BitU8, ...]`.
#[derive(Copy, Clone, Debug)]
pub struct BitU8(pub u8);

/// A built-in Zdex-compatible `BitCollection` for `u8`.
#[bit(BitU8, mask = "!0", retr = "0")]
#[derive(Copy, Clone, BitCollection, Debug)]
pub struct FromU8(u8);

/// A built-in Zdex-compatible `BitCollection` for `u16`.
#[bit(BitU8, mask = "!0", retr = "0")]
#[derive(Copy, Clone, BitCollection, Debug)]
pub struct FromU16(u16);

/// A built-in Zdex-compatible `BitCollection` for `u32`.
#[bit(BitU8, mask = "!0", retr = "0")]
#[derive(Copy, Clone, BitCollection, Debug)]
pub struct FromU32(u32);

/// A built-in Zdex-compatible `BitCollection` for `u64`.
#[bit(BitU8, mask = "!0", retr = "0")]
#[derive(Copy, Clone, BitCollection, Debug)]
pub struct FromU64(u64);

/// A built-in Zdex-compatible `BitCollection` for `u128`.
#[bit(BitU8, mask = "!0", retr = "0")]
#[derive(Copy, Clone, BitCollection, Debug)]
pub struct FromU128(u128);


#[cfg(test)]
mod tests {
    use crate::*;
    use vob::vob;

    #[test]
    fn it_works() -> Result<(), std::io::Error> {
        let v: FromU8 = 0b10000101.into();
        assert_eq!(FromU8(0b11).z_index()?, vob![true, true]);
        assert_eq!(
            v.z_index()?,
            vob![true, false, false, false, false, true, false, true]
        );
        Ok(())
    }

    #[test]
    fn iter_works() -> Result<(), std::io::Error> {
        let v1: FromU8 = 0b010.into();
        let v2: FromU8 = 0b100.into();
        assert_eq!(
            vec![v1, v2].z_index()?,
            vob![false, true, true, false, false, false]
        );

        Ok(())
    }

    #[test]
    fn tupl_works() -> Result<(), std::io::Error> {
        let v1: FromU8 = 0b100000.into();
        let v2: FromU8 = 0b000010.into();
        let v3: FromU8 =     0b10.into();

        assert_eq!(
            (v1, v2).z_index()?,
            vob![true, false, false, false, false, false, false, false,
                false, true, false, false]
        );

        assert_eq!(
            (v1, v2, v3).z_index()?,
            vob![true, false, false,
                false, false, false,
                false, false, false,
                false, false, false,
                false,  true,  true,
                false, false, false]
        );

        Ok(())
    }

    #[test]
    fn heterogeneous_tuples() -> Result<(), std::io::Error> {
        let my_u8:  FromU8  = 0xf0.into();
        let my_u16: FromU16 = 0xff00.into();

        assert_eq!(
            (my_u8, my_u16)
                .z_index()?
                .iter_storage()
                .next()
                .expect("empty vob"),
            //   0b0000_0000_1111_0000
            // z 0b1111_1111_0000_0000
            // ==
            //   0b00000000_00000000_10101010_00000000
            // ^ 0b01010101_01010101_00000000_00000000
            0b_____01010101_01010101_10101010_00000000
        );

        Ok(())
    }
}
