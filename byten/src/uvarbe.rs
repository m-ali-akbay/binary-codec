#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;

#[cfg(feature = "alloc")]
use crate::U8Codec;
use crate::error::DecodeError;

pub type BitIndex = usize;

pub trait BitStream: Sized {
    const BITS: usize;

    fn to_bits(&self) -> impl Iterator<Item = BitIndex>;
    fn try_from_bits(
        bits: impl Iterator<Item = BitIndex>,
    ) -> Result<Self, crate::error::DecodeError>;
}

macro_rules! u_bit_stream {
    ($($ty:tt),*) => {
        $(impl BitStream for $ty {
            const BITS: usize = $ty::BITS as usize;

            fn to_bits(&self) -> impl Iterator<Item = BitIndex> {
                (0..Self::BITS as usize).filter_map(move |bit_index| {
                    if (self & (1 << bit_index)) != 0 {
                        Some(bit_index)
                    } else {
                        None
                    }
                })
            }

            fn try_from_bits(bits: impl Iterator<Item = BitIndex>) -> Result<Self, DecodeError> {
                let mut value: $ty = 0;
                for bit_index in bits {
                    if bit_index >= Self::BITS as usize {
                        return Err(DecodeError::BitOverflow);
                    }
                    value |= 1 << bit_index;
                }
                Ok(value)
            }

        })*
    }
}
u_bit_stream!(u16, u32, u64, u128, usize);

/// A codec for unsigned variable-length big-endian encoded integers.
/// The integer is represented as a series of 7-bit septets, where the most significant bit of each septet
/// indicates whether there are more septets to follow.
///
/// The septets are stored in big-endian order, meaning the most significant septet comes first.
/// This allows comparison of encoded values using standard lexicographical byte-wise comparison without decoding.
///
/// This codec supports integer types that implement the `BitStream` trait.
///
/// Minimum size is 1 byte (for value 0), maximum size depends on the number of bits in the type.
/// As the size is variable, there is no fixed size measurement.
/// The compressions achieved by this codec is beneficial when smaller values are more common by truncating
/// leading (most significant bits) zero septets.
///
/// This codec is useful for encoding integers in a compact form, especially when smaller values are more common.
/// Also, useful for coding the length of slices, vectors, or other data structures where the length is not known in advance.
///
/// # Examples
/// ```rust
/// use byten::{UVarBECodec, Encoder, Decoder, Measurer, EncoderToVec as _};
///
/// let codec = UVarBECodec::<u64>::new();
/// let value: u64 = 0x123456;
///
/// let mut encoded = codec.encode_to_vec(&value).unwrap();
///
/// let mut decode_offset = 0;
/// let decoded = codec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, value);
///
/// let size = codec.measure(&value).unwrap();
/// assert_eq!(size, encoded.len());
/// ```
#[derive(Copy, Clone)]
pub struct UVarBECodec<T> {
    pub _marker: PhantomData<T>,
}

impl<T> UVarBECodec<T> {
    pub const fn new() -> Self {
        UVarBECodec {
            _marker: PhantomData,
        }
    }
}

#[cfg(feature = "alloc")]
impl<T: BitStream> UVarBECodec<T> {
    fn try_into_septets_le(num: &T) -> Result<Vec<u8>, crate::error::EncodeError> {
        let septets = T::BITS / 7usize + if T::BITS % 7usize == 0 { 0 } else { 1 };
        let mut septets: Vec<u8> = vec![0u8; septets];

        for bit_index in num.to_bits() {
            let septet_index = bit_index / 7;
            let septet = septets
                .get_mut(septet_index)
                .ok_or(crate::error::EncodeError::BitOverflow)?;
            let septet_bit_index = bit_index % 7;
            *septet |= 1 << septet_bit_index;
        }

        Ok(septets)
    }

    #[cfg(test)]
    fn try_into_septets_be(num: &T) -> Result<Vec<u8>, crate::error::EncodeError> {
        let mut septets_le = Self::try_into_septets_le(num)?;
        septets_le.reverse();
        Ok(septets_le)
    }

    fn try_from_septets_le(
        septets_le: impl Iterator<Item = u8>,
    ) -> Result<T, crate::error::DecodeError> {
        let bits = septets_le.flat_map(|septet| {
            (0..7).map(move |septet_bit_index| septet & (1 << septet_bit_index) != 0)
        });
        let bits = (0..).zip(bits).filter_map(|(bit_index, bit)| match bit {
            true => Some(bit_index),
            false => None,
        });
        T::try_from_bits(bits)
    }

    fn try_from_septets_be(
        septets_be: impl DoubleEndedIterator<Item = u8>,
    ) -> Result<T, crate::error::DecodeError> {
        Self::try_from_septets_le(septets_be.rev())
    }
}

#[cfg(feature = "alloc")]
impl<T: BitStream> crate::Encoder for UVarBECodec<T> {
    type Decoded = T;
    fn encode(
        &self,
        decoded: &T,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), crate::error::EncodeError> {
        let septets_le = Self::try_into_septets_le(decoded)?;

        let trunc = septets_le.iter().rev().take_while(|&&b| b == 0).count();
        let full = septets_le.len() - trunc;

        let septets_le = &septets_le[..full];

        if septets_le.is_empty() {
            return U8Codec.encode(&0, encoded, offset);
        }

        for &septet in septets_le[1..].iter().rev() {
            U8Codec.encode(&(septet | 0x80), encoded, offset)?;
        }

        U8Codec.encode(&septets_le[0], encoded, offset)?;

        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<T: BitStream> crate::Measurer for UVarBECodec<T> {
    type Decoded = T;
    fn measure(&self, decoded: &T) -> Result<usize, crate::error::EncodeError> {
        let septets_le = Self::try_into_septets_le(decoded)?;

        let trunc = septets_le.iter().rev().take_while(|&&b| b == 0).count();
        let full = septets_le.len() - trunc;

        let septets_le = &septets_le[0..full];

        Ok(septets_le.len().max(1))
    }
}

#[cfg(feature = "alloc")]
impl<'encoded, 'decoded, T: BitStream + 'decoded> crate::Decoder<'encoded, 'decoded>
    for UVarBECodec<T>
{
    type Decoded = T;

    fn decode(
        &self,
        encoded: &'encoded [u8],
        offset: &mut usize,
    ) -> Result<T, crate::error::DecodeError> {
        let max_septets = T::BITS / 7usize + if T::BITS % 7usize == 0 { 0 } else { 1 };
        let mut septets_be: Vec<u8> = Vec::with_capacity(max_septets);
        for i in 0.. {
            if i >= max_septets {
                return Err(crate::error::DecodeError::BitOverflow);
            }
            let flagged_septet = U8Codec.decode(encoded, offset)?;
            let flag = flagged_septet & 0x80;
            let septet = flagged_septet & 0x7F;
            septets_be.push(septet);
            if flag == 0 {
                break;
            }
        }

        Self::try_from_septets_be(septets_be.into_iter())
    }
}

#[cfg(test)]
mod test {
    use crate::Decoder as _;
    use crate::prelude::EncoderToVec;

    use super::*;

    #[test]
    fn test_septets_le() {
        let fixtures = [
            (
                0u64,
                [
                    0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000,
                    0b0000000, 0b0000000, 0b0000000,
                ],
            ),
            (
                1u64,
                [
                    0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000,
                    0b0000000, 0b0000000, 0b0000001,
                ],
            ),
            (
                127u64,
                [
                    0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000,
                    0b0000000, 0b0000000, 0b1111111,
                ],
            ),
            (
                128u64,
                [
                    0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000,
                    0b0000000, 0b0000001, 0b0000000,
                ],
            ),
            (
                255u64,
                [
                    0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000,
                    0b0000000, 0b0000001, 0b1111111,
                ],
            ),
            (
                16383u64,
                [
                    0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000,
                    0b0000000, 0b1111111, 0b1111111,
                ],
            ),
            (
                16384u64,
                [
                    0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000,
                    0b0000001, 0b0000000, 0b0000000,
                ],
            ),
            (
                u64::MAX,
                [
                    0b0000001, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111,
                    0b1111111, 0b1111111, 0b1111111,
                ],
            ),
        ];
        for (num, septets_be_fixture) in fixtures.iter() {
            let septets_be = UVarBECodec::try_into_septets_be(num).unwrap();
            assert_eq!(
                &septets_be, septets_be_fixture,
                "BE septets failed for {}",
                num
            );

            let reconstructed_num: u64 =
                UVarBECodec::try_from_septets_be((*septets_be_fixture).into_iter()).unwrap();
            assert_eq!(
                &reconstructed_num, num,
                "BE reconstruction failed for {:?}",
                septets_be_fixture
            );
        }
    }

    #[test]
    fn test_uvarbe() {
        let fixtures = [
            (0u64, vec![0b00000000]),
            (1u64, vec![0b00000001]),
            (127u64, vec![0b01111111]),
            (128u64, vec![0b10000001, 0b00000000]),
            (255u64, vec![0b10000001, 0b01111111]),
            (16383u64, vec![0b11111111, 0b01111111]),
            (16384u64, vec![0b10000001, 0b10000000, 0b00000000]),
            (
                u64::MAX,
                vec![
                    0b10000001, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
                    0b11111111, 0b11111111, 0b11111111, 0b01111111,
                ],
            ),
        ];

        for (num, encoded_fixture) in fixtures.iter() {
            let encoded = UVarBECodec::new()
                .encode_to_vec(num)
                .expect("Encoding failed");
            assert_eq!(&encoded, encoded_fixture, "Encoding failed for {}", num);

            let decoded: u64 = UVarBECodec::new()
                .decode(&encoded, &mut 0)
                .expect("Decoding failed");
            assert_eq!(&decoded, num, "Decoding failed for {:?}", encoded);
        }
    }
}
