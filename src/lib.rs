//! Zdex is for evaluating z-order indexes (morton encoding) for types,
//! iterators, and tuples of
//! [BitCollections](https://crates.io/crates/bit_collection).
//!
//! Z-order indexing is a database range-querying technique to optimise
//! scanning performance, and Zdex aims to be prescriptive in providing that
//! functionality.

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

impl<T> ZdexedTup for (T, T)
    where T: Zdexed
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
}
