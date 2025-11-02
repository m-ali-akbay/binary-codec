use std::{marker::PhantomData, option::Option as StdOption, vec::Vec as StdVec};

use crate::{BoolCodec, DecodeError, U8Codec};

pub struct VecCodec<Item, Length> {
    pub item: Item,
    pub length: Length,
}

impl<Item, Length> VecCodec<Item, Length> {
    pub const fn new(item: Item, length: Length) -> Self {
        Self { item, length }
    }
}

impl<'encoded, 'decoded, 'length, Item, Length> crate::Decoder<'encoded, 'decoded> for VecCodec<Item, Length>
where
    Item: crate::Decoder<'encoded, 'decoded>,
    Length: crate::Decoder<'encoded, 'length>,
    Length::Decoded: TryInto<usize>,
    <Length::Decoded as TryInto<usize>>::Error: Into<crate::DecodeError>,
{
    type Decoded = StdVec<Item::Decoded>;

    fn decode(&self, encoded: &'encoded [u8], offset: &mut usize) -> Result<Self::Decoded, crate::DecodeError> {
        let size = self.length.decode(encoded, offset)?.try_into().map_err(Into::into)?;
        let mut vec = StdVec::with_capacity(size);
        for _ in 0..size {
            let item = self.item.decode(encoded, offset)?;
            vec.push(item);
        }
        Ok(vec)
    }
}

impl<Item, Length> crate::Encoder for VecCodec<Item, Length>
where
    Length: crate::Encoder,
    Length::Decoded: TryFrom<usize>,
    <Length::Decoded as TryFrom<usize>>::Error: Into<crate::EncodeError>,
    Item: crate::Encoder,
    Item::Decoded: Sized,
{
    type Decoded = StdVec<Item::Decoded>;

    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), crate::EncodeError> {
        let size = decoded.len();
        self.length.encode(&size.try_into().map_err(Into::into)?, encoded, offset)?;
        for item in decoded.iter() {
            self.item.encode(item, encoded, offset)?;
        }
        Ok(())
    }
}

impl<Item, Length> crate::Measurer for VecCodec<Item, Length>
where
    Length: crate::Measurer,
    Length::Decoded: TryFrom<usize>,
    <Length::Decoded as TryFrom<usize>>::Error: Into<crate::EncodeError>,
    Item: crate::Measurer,
    Item::Decoded: Sized,
{
    type Decoded = StdVec<Item::Decoded>;

    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, crate::EncodeError> {
        let size = decoded.len();
        let size_measure = self.length.measure(&size.try_into().map_err(Into::into)?)?;
        let mut items_measure = 0;
        for item in decoded.iter() {
            items_measure += self.item.measure(item)?;
        }
        Ok(size_measure + items_measure)
    }
}

pub struct RemainingCodec;

impl RemainingCodec {
    pub const fn new() -> Self {
        RemainingCodec
    }
}

impl<'encoded, 'decoded> crate::Decoder<'encoded, 'decoded> for RemainingCodec
where
    'encoded: 'decoded,
{
    type Decoded = &'decoded [u8];

    fn decode(&self, encoded: &'encoded [u8], offset: &mut usize) -> Result<Self::Decoded, crate::DecodeError> {
        if *offset > encoded.len() {
            return Err(crate::DecodeError::InvalidData);
        }
        let remaining = &encoded[*offset..];
        *offset = encoded.len();
        Ok(remaining)
    }
}

impl crate::Encoder for RemainingCodec {
    type Decoded = [u8];

    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), crate::EncodeError> {
        let end = *offset + decoded.len();
        if end > encoded.len() {
            return Err(crate::EncodeError::BufferTooSmall);
        }
        encoded[*offset..end].copy_from_slice(decoded);
        *offset = end;
        Ok(())
    }
}

impl crate::Measurer for RemainingCodec {
    type Decoded = [u8];

    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, crate::EncodeError> {
        Ok(decoded.len())
    }
}

pub type BitIndex = usize;

pub trait BitStream: Sized {
    const BITS: usize;

    fn to_bits(&self) -> impl Iterator<Item = BitIndex>;
    fn try_from_bits(bits: impl Iterator<Item = BitIndex>) -> Result<Self, DecodeError>;
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

#[derive(Copy, Clone)]
pub struct UVarBECodec<T> {
    pub _marker: PhantomData<T>,
}

impl<T> UVarBECodec<T> {
    pub const fn new() -> Self { UVarBECodec { _marker: PhantomData } }
}

impl<T: BitStream> UVarBECodec<T> {
    fn try_into_septets_le(num: &T) -> Result<Vec<u8>, crate::EncodeError> {
        let septets = T::BITS / 7usize + if T::BITS % 7usize == 0 { 0 } else { 1 };
        let mut septets: Vec<u8> = vec![0u8; septets];

        for bit_index in num.to_bits() {
            let septet_index = bit_index / 7;
            let septet = septets.get_mut(septet_index).ok_or(crate::EncodeError::BitOverflow)?;
            let septet_bit_index = bit_index % 7;
            *septet = (*septet) | (1 << septet_bit_index);
        }

        Ok(septets)
    }

    #[cfg(test)]
    fn try_into_septets_be(num: &T) -> Result<Vec<u8>, crate::EncodeError> {
        let mut septets_le = Self::try_into_septets_le(num)?;
        septets_le.reverse();
        Ok(septets_le)
    }

    fn try_from_septets_le(septets_le: impl Iterator<Item = u8>) -> Result<T, crate::DecodeError> {
        let bits = septets_le.flat_map(
            |septet| (0..7).map(move |septet_bit_index|septet & (1 << septet_bit_index) != 0)
        );
        let bits = (0..).zip(bits).filter_map(
            |(bit_index, bit)| match bit {
                true => Some(bit_index),
                false => None,
            }
        );
        T::try_from_bits(bits)
    }

    fn try_from_septets_be(septets_be: impl DoubleEndedIterator<Item = u8>) -> Result<T, crate::DecodeError> {
        Self::try_from_septets_le(septets_be.rev())
    }
}

impl<T: BitStream> crate::Encoder for UVarBECodec<T> {
    type Decoded = T;
    fn encode(&self, decoded: &T, encoded: &mut [u8], offset: &mut usize) -> Result<(), crate::EncodeError> {
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

impl<T: BitStream> crate::Measurer for UVarBECodec<T> {
    type Decoded = T;
    fn measure(&self, decoded: &T) -> Result<usize, crate::EncodeError> {
        let septets_le = Self::try_into_septets_le(decoded)?;

        let trunc = septets_le.iter().rev().take_while(|&&b| b == 0).count();
        let full = septets_le.len() - trunc;

        let septets_le = &septets_le[0..full];

        Ok(septets_le.len().max(1))
    }
}

impl<'encoded, 'decoded, T: BitStream + 'decoded> crate::Decoder<'encoded, 'decoded> for UVarBECodec<T> {
    type Decoded = T;

    fn decode(&self, encoded: &'encoded [u8], offset: &mut usize) -> Result<T, crate::DecodeError> {
        let max_septets = T::BITS / 7usize + if T::BITS % 7usize == 0 { 0 } else { 1 };
        let mut septets_be: Vec<u8> = Vec::with_capacity(max_septets);
        for i in 0.. {
            if i >= max_septets {
                return Err(crate::DecodeError::BitOverflow);
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
    use crate::prelude::EncoderToVec;
    use crate::Decoder as _;

    use super::*;

    #[test]
    fn test_septets_le() {
        let fixtures = [
            (0u64,      [0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000]),
            (1u64,      [0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000001]),
            (127u64,    [0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b1111111]),
            (128u64,    [0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000001, 0b0000000]),
            (255u64,    [0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000001, 0b1111111]),
            (16383u64,  [0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b1111111, 0b1111111]),
            (16384u64,  [0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b0000001, 0b0000000, 0b0000000]),
            (u64::MAX,  [0b0000001, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111]),
        ];
        for (num, septets_be_fixture) in fixtures.iter() {
            let septets_be = UVarBECodec::try_into_septets_be(num).unwrap();
            assert_eq!(&septets_be, septets_be_fixture, "BE septets failed for {}", num);

            let reconstructed_num: u64 = UVarBECodec::try_from_septets_be(septets_be_fixture.clone().into_iter()).unwrap();
            assert_eq!(&reconstructed_num, num, "BE reconstruction failed for {:?}", septets_be_fixture);
        }
    }
    
    #[test]
    fn test_uvarbe() {
        let fixtures = [
            (0u64,      vec![0b00000000]),
            (1u64,      vec![0b00000001]),
            (127u64,    vec![0b01111111]),
            (128u64,    vec![0b10000001, 0b00000000]),
            (255u64,    vec![0b10000001, 0b01111111]),
            (16383u64,  vec![0b11111111, 0b01111111]),
            (16384u64,  vec![0b10000001, 0b10000000, 0b00000000]),
            (u64::MAX,  vec![0b10000001, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b01111111]),
        ];

        for (num, encoded_fixture) in fixtures.iter() {
            let encoded = UVarBECodec::new().encode_to_vec(num).expect("Encoding failed");
            assert_eq!(&encoded, encoded_fixture, "Encoding failed for {}", num);

            let decoded: u64 = UVarBECodec::new().decode(&encoded, &mut 0).expect("Decoding failed");
            assert_eq!(&decoded, num, "Decoding failed for {:?}", encoded);
        }
    }
}

pub struct OptionCodec<Item> {
    pub item: Item,
}

impl<Item> OptionCodec<Item> {
    pub fn new(item: Item) -> Self {
        Self { item }
    }
}

impl<'encoded, 'decoded, Item> crate::Decoder<'encoded, 'decoded> for OptionCodec<Item>
where
    Item: crate::Decoder<'encoded, 'decoded>,
{
    type Decoded = StdOption<Item::Decoded>;

    fn decode(&self, encoded: &'encoded [u8], offset: &mut usize) -> Result<Self::Decoded, crate::DecodeError> {
        let flag = BoolCodec.decode(encoded, offset)?;
        if flag {
            Ok(StdOption::None)
        } else {
            let item = self.item.decode(encoded, offset)?;
            Ok(StdOption::Some(item))
        }
    }
}

impl<Item> crate::Encoder for OptionCodec<Item>
where
    Item: crate::Encoder,
    Item::Decoded: Sized,
{
    type Decoded = StdOption<Item::Decoded>;

    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), crate::EncodeError> {
        match decoded {
            StdOption::None => {
                BoolCodec.encode(&true, encoded, offset)?;
                Ok(())
            }
            StdOption::Some(item) => {
                BoolCodec.encode(&false, encoded, offset)?;
                self.item.encode(item, encoded, offset)
            }
        }
    }
}

impl<Item> crate::Measurer for OptionCodec<Item>
where
    Item: crate::Measurer,
    Item::Decoded: Sized,
{
    type Decoded = StdOption<Item::Decoded>;

    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, crate::EncodeError> {
        Ok(match decoded {
            StdOption::None => BoolCodec.measure(&true)?,
            StdOption::Some(item) => {
                BoolCodec.measure(&false)?
                + self.item.measure(item)?
            }
        })
    }
}

pub struct BytesCodec<Length> {
    pub length: Length,
}

impl<Length> BytesCodec<Length> {
    pub const fn new(length: Length) -> Self {
        Self {
            length,
        }
    }
}

impl<'encoded, 'decoded, 'length, Length> crate::Decoder<'encoded, 'decoded> for BytesCodec<Length>
where
    Length: crate::Decoder<'encoded, 'length>,
    Length::Decoded: TryInto<usize>,
    <Length::Decoded as TryInto<usize>>::Error: Into<crate::DecodeError>,
    'encoded: 'decoded,
{
    type Decoded = &'decoded [u8];

    fn decode(&self, encoded: &'encoded [u8], offset: &mut usize) -> Result<Self::Decoded, crate::DecodeError> {
        let size = self.length.decode(encoded, offset)?.try_into().map_err(Into::into)?;
        if *offset + size > encoded.len() {
            return Err(crate::DecodeError::InvalidData);
        }
        let buffer = &encoded[*offset..*offset + size];
        *offset += size;
        Ok(buffer)
    }
}

impl<Length> crate::Encoder for BytesCodec<Length>
where
    Length: crate::Encoder,
    Length::Decoded: TryFrom<usize>,
    <Length::Decoded as TryFrom<usize>>::Error: Into<crate::EncodeError>,
{
    type Decoded = [u8];

    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), crate::EncodeError> {
        let size = decoded.len();
        self.length.encode(&size.try_into().map_err(Into::into)?, encoded, offset)?;
        let end = *offset + size;
        if end > encoded.len() {
            return Err(crate::EncodeError::BufferTooSmall);
        }
        encoded[*offset..end].copy_from_slice(decoded);
        *offset = end;
        Ok(())
    }
}

impl<Length> crate::Measurer for BytesCodec<Length>
where
    Length: crate::Measurer,
    Length::Decoded: TryFrom<usize>,
    <Length::Decoded as TryFrom<usize>>::Error: Into<crate::EncodeError>,
{
    type Decoded = [u8];

    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, crate::EncodeError> {
        let size = decoded.len();
        let size_measure = self.length.measure(&size.try_into().map_err(Into::into)?)?;
        Ok(size_measure + size)
    }
}
