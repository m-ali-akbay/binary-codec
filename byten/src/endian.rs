use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Endianness {
    Big,
    Little,
}

/// A trait for types that can encode and decode themselves with specified endianness.
/// The implementor must ensure that the length of the byte representation is constant.
///
/// The intention is to support primitive types like u16, i32, etc, and not to be implemented
/// by user-defined types.
///
/// # Examples
/// ## Decoding and encoding a u32 value
/// ```rust
/// use byten::{Encoder, Decoder, Measurer, FixedMeasurer, Endianness, EndianCodec};
///
/// let value: u32 = 0x12345678;
/// let codec = EndianCodec::new(Endianness::Little);
///
/// let mut encoded = [0u8; 4];
/// let mut offset = 0;
/// codec.encode(&value, &mut encoded, &mut offset).unwrap();
/// assert_eq!(offset, 4);
/// assert_eq!(encoded, [0x78, 0x56, 0x34, 0x12]);
///
/// let mut decode_offset = 0;
/// let decoded: u32 = codec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, value);
/// assert_eq!(decode_offset, 4);
///
/// let size = codec.measure(&value).unwrap();
/// assert_eq!(size, 4);
///
/// let fixed_size = codec.measure_fixed();
/// assert_eq!(fixed_size, 4);
/// ```
///
/// ## Internal use
/// The use below is for illustration only. Users should use the `Decoder`, `Encoder`, `Measurer`, and `FixedMeasurer` traits directly.
/// ```rust
/// use byten::{EndianCoded as _, Endianness};
///
/// let value: u32 = 0x12345678;
///
/// let mut bytes = [0u8; 4];
/// value.encode(Endianness::Little, &mut bytes);
/// assert_eq!(bytes, [0x78, 0x56, 0x34, 0x12]);
///
/// let decoded = u32::decode(Endianness::Little, &bytes);
/// assert_eq!(decoded, value);
/// ```
pub trait EndianCoded {
    fn length() -> usize;

    fn decode(endianness: Endianness, bytes: &[u8]) -> Self;
    fn encode(&self, endianness: Endianness, bytes: &mut [u8]);
}

/// A codec that encodes and decodes values using specified endianness.
///
/// # Examples
/// ```rust
/// use byten::{EndianCodec, Endianness, Decoder, Encoder, Measurer, FixedMeasurer};
///
/// let codec = EndianCodec::<u32>::le();
/// let value: u32 = 0x12345678;
///
/// let mut encoded = [0u8; 4];
/// let mut offset = 0;
/// codec.encode(&value, &mut encoded, &mut offset).unwrap();
/// assert_eq!(encoded, [0x78, 0x56, 0x34, 0x12]);
///
/// let mut decode_offset = 0;
/// let decoded: u32 = codec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, value);
///
/// let size = codec.measure(&value).unwrap();
/// assert_eq!(size, 4);
///
/// let fixed_size = codec.measure_fixed();
/// assert_eq!(fixed_size, 4);
/// ```
pub struct EndianCodec<T> {
    pub endianness: Endianness,
    _marker: PhantomData<T>,
}

impl<T> EndianCodec<T> {
    pub const fn new(endianness: Endianness) -> Self {
        Self {
            endianness,
            _marker: PhantomData,
        }
    }

    pub const fn le() -> Self {
        Self::new(Endianness::Little)
    }

    pub const fn be() -> Self {
        Self::new(Endianness::Big)
    }
}

impl<'decoded, T: EndianCoded + 'decoded> crate::Decoder<'_, 'decoded> for EndianCodec<T> {
    type Decoded = T;

    fn decode(
        &self,
        encoded: &'_ [u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, crate::DecodeError> {
        let length = T::length();
        if *offset + length > encoded.len() {
            return Err(crate::DecodeError::EOF);
        }
        let value = T::decode(self.endianness, &encoded[*offset..*offset + length]);
        *offset += length;
        Ok(value)
    }
}

impl<T: EndianCoded> crate::Encoder for EndianCodec<T> {
    type Decoded = T;

    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), crate::EncodeError> {
        let length = T::length();
        if *offset + length > encoded.len() {
            return Err(crate::EncodeError::BufferTooSmall);
        }
        decoded.encode(self.endianness, &mut encoded[*offset..*offset + length]);
        *offset += length;
        Ok(())
    }
}

impl<T: EndianCoded> crate::FixedMeasurer for EndianCodec<T> {
    fn measure_fixed(&self) -> usize {
        T::length()
    }
}

impl<T: EndianCoded> crate::Measurer for EndianCodec<T> {
    type Decoded = T;

    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, crate::error::EncodeError> {
        Ok(crate::FixedMeasurer::measure_fixed(self))
    }
}

macro_rules! impl_endian_coded {
    ($($ty:ident),* $(,)?) => {
        $(impl EndianCoded for $ty {
            fn length() -> usize {
                (<$ty>::BITS as usize) / 8
            }

            fn decode(endianness: Endianness, bytes: &[u8]) -> Self {
                match endianness {
                    Endianness::Big => <$ty>::from_be_bytes(bytes.try_into().unwrap()),
                    Endianness::Little => <$ty>::from_le_bytes(bytes.try_into().unwrap()),
                }
            }

            fn encode(&self, endianness: Endianness, bytes: &mut [u8]) {
                let byte_array = match endianness {
                    Endianness::Big => self.to_be_bytes(),
                    Endianness::Little => self.to_le_bytes(),
                };
                bytes.copy_from_slice(&byte_array);
            }
        })*
    };
}

impl_endian_coded!(u16, i16, u32, i32, u64, i64, u128, i128,);
