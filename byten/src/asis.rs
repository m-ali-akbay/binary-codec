//! A set of codecs for primitive types and fixed-size arrays.
//!
//! All the codecs in this module must include no implicit logic
//! beyond the binary representation of the types they handle.

use crate::{Decoder, Encoder, FixedMeasurer, Measurer, error};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// A codec for the `u8` type.
///
/// # Examples
/// ```rust
/// use byten::{U8Codec, Decoder, Encoder, FixedMeasurer, Measurer, DecodeError, EncodeError};
///
/// let value: u8 = 42;
///
/// let mut encoded = [0u8; 1];
/// let mut offset = 0;
/// U8Codec.encode(&value, &mut encoded, &mut offset).unwrap();
/// assert_eq!(offset, 1);
///
/// let mut decode_offset = 0;
/// let decoded: u8 = U8Codec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, value);
/// assert_eq!(decode_offset, 1);
///
/// let size = U8Codec.measure(&value).unwrap();
/// assert_eq!(size, 1);
///
/// let fixed_size = U8Codec.measure_fixed();
/// assert_eq!(fixed_size, 1);
/// ```
pub struct U8Codec;

impl U8Codec {
    pub const fn new() -> Self {
        Self
    }
}

impl Decoder<'_, '_> for U8Codec {
    type Decoded = u8;
    fn decode(
        &self,
        encoded: &[u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, error::DecodeError> {
        if *offset + 1 > encoded.len() {
            return Err(error::DecodeError::EOF);
        }
        let byte = encoded[*offset];
        *offset += 1;
        Ok(byte)
    }
}

impl Encoder for U8Codec {
    type Decoded = u8;
    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), error::EncodeError> {
        if *offset + 1 > encoded.len() {
            return Err(error::EncodeError::BufferTooSmall);
        }
        encoded[*offset] = *decoded;
        *offset += 1;
        Ok(())
    }
}

impl FixedMeasurer for U8Codec {
    fn measure_fixed(&self) -> usize {
        1
    }
}

impl Measurer for U8Codec {
    type Decoded = u8;
    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, error::EncodeError> {
        Ok(self.measure_fixed())
    }
}

/// A codec for the `i8` type.
///
/// # Examples
/// ```rust
/// use byten::{I8Codec, Decoder, Encoder, FixedMeasurer, Measurer, DecodeError, EncodeError};
///
/// let value: i8 = -42;
///
/// let mut encoded = [0u8; 1];
/// let mut offset = 0;
/// I8Codec.encode(&value, &mut encoded, &mut offset).unwrap();
/// assert_eq!(offset, 1);
///
/// let mut decode_offset = 0;
/// let decoded: i8 = I8Codec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, value);
/// assert_eq!(decode_offset, 1);
///
/// let size = I8Codec.measure(&value).unwrap();
/// assert_eq!(size, 1);
///
/// let fixed_size = I8Codec.measure_fixed();
/// assert_eq!(fixed_size, 1);
/// ```
pub struct I8Codec;

impl I8Codec {
    pub const fn new() -> Self {
        Self
    }
}

impl Decoder<'_, '_> for I8Codec {
    type Decoded = i8;
    fn decode(
        &self,
        encoded: &[u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, error::DecodeError> {
        if *offset + 1 > encoded.len() {
            return Err(error::DecodeError::EOF);
        }
        let byte = encoded[*offset] as i8;
        *offset += 1;
        Ok(byte)
    }
}

impl Encoder for I8Codec {
    type Decoded = i8;
    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), error::EncodeError> {
        if *offset + 1 > encoded.len() {
            return Err(error::EncodeError::BufferTooSmall);
        }
        encoded[*offset] = *decoded as u8;
        *offset += 1;
        Ok(())
    }
}

impl FixedMeasurer for I8Codec {
    fn measure_fixed(&self) -> usize {
        1
    }
}

impl Measurer for I8Codec {
    type Decoded = i8;
    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, error::EncodeError> {
        Ok(self.measure_fixed())
    }
}

/// A codec for fixed-size arrays of `u8`.
///
/// # Examples
/// ```rust
/// use byten::{U8ArrayCodec, Decoder, Encoder, FixedMeasurer, Measurer, DecodeError, EncodeError};
///
/// let array: [u8; 4] = [1, 2, 3, 4];
/// let mut encoded = [0u8; 4];
/// let mut offset = 0;
///
/// U8ArrayCodec::<4>.encode(&array, &mut encoded, &mut offset).unwrap();
/// assert_eq!(offset, 4);
///
/// let mut decode_offset = 0;
/// let decoded: [u8; 4] = U8ArrayCodec::<4>.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, array);
/// assert_eq!(decode_offset, 4);
///
/// let size = U8ArrayCodec::<4>.measure(&array).unwrap();
/// assert_eq!(size, 4);
///
/// let fixed_size = U8ArrayCodec::<4>.measure_fixed();
/// assert_eq!(fixed_size, 4);
/// ```
pub struct U8ArrayCodec<const N: usize>;

impl<const N: usize> U8ArrayCodec<N> {
    pub const fn new() -> Self {
        Self
    }
}

impl<const N: usize> Decoder<'_, '_> for U8ArrayCodec<N> {
    type Decoded = [u8; N];
    fn decode(
        &self,
        encoded: &[u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, error::DecodeError> {
        U8ArrayRefCodec::<N>.decode(encoded, offset).copied()
    }
}

impl<const N: usize> Encoder for U8ArrayCodec<N> {
    type Decoded = [u8; N];
    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), error::EncodeError> {
        U8ArrayRefCodec::<N>.encode(decoded, encoded, offset)
    }
}

impl<const N: usize> FixedMeasurer for U8ArrayCodec<N> {
    fn measure_fixed(&self) -> usize {
        N
    }
}

impl<const N: usize> Measurer for U8ArrayCodec<N> {
    type Decoded = [u8; N];
    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, error::EncodeError> {
        Ok(self.measure_fixed())
    }
}

/// A codec for references to fixed-size arrays of `u8`.
///
/// # Examples
/// ```rust
/// use byten::{U8ArrayRefCodec, Decoder, Encoder, FixedMeasurer, Measurer, DecodeError, EncodeError};
///
/// let array_ref: &[u8; 4] = &[1, 2, 3, 4];
///
/// let mut encoded = [0u8; 4];
/// let mut offset = 0;
/// U8ArrayRefCodec.encode(array_ref, &mut encoded, &mut offset).unwrap();
/// assert_eq!(offset, 4);
///
/// let mut decode_offset = 0;
/// let decoded: &[u8; 4] = U8ArrayRefCodec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, array_ref);
/// assert_eq!(decode_offset, 4);
///
/// let size = U8ArrayRefCodec::<4>.measure(array_ref).unwrap();
/// assert_eq!(size, 4);
///
/// let fixed_size = U8ArrayRefCodec::<4>.measure_fixed();
/// assert_eq!(fixed_size, 4);
/// ```
pub struct U8ArrayRefCodec<const N: usize>;

impl<const N: usize> U8ArrayRefCodec<N> {
    pub const fn new() -> Self {
        Self
    }
}

impl<'encoded: 'decoded, 'decoded, const N: usize> Decoder<'encoded, 'decoded>
    for U8ArrayRefCodec<N>
{
    type Decoded = &'decoded [u8; N];
    fn decode(
        &self,
        encoded: &'encoded [u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, error::DecodeError> {
        if *offset + N > encoded.len() {
            return Err(error::DecodeError::EOF);
        }
        let array = encoded[*offset..*offset + N].try_into().unwrap();
        *offset += N;
        Ok(array)
    }
}

impl<const N: usize> Encoder for U8ArrayRefCodec<N> {
    type Decoded = [u8; N];
    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), error::EncodeError> {
        if *offset + N > encoded.len() {
            return Err(error::EncodeError::BufferTooSmall);
        }
        encoded[*offset..*offset + N].copy_from_slice(decoded);
        *offset += N;
        Ok(())
    }
}

impl<const N: usize> FixedMeasurer for U8ArrayRefCodec<N> {
    fn measure_fixed(&self) -> usize {
        N
    }
}

impl<const N: usize> Measurer for U8ArrayRefCodec<N> {
    type Decoded = [u8; N];
    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, error::EncodeError> {
        Ok(self.measure_fixed())
    }
}

/// A codec for the `bool` type.
///
/// # Examples
/// ```rust
/// use byten::{BoolCodec, Decoder, Encoder, FixedMeasurer, Measurer, DecodeError, EncodeError};
///
/// let value: bool = true;
///
/// let mut encoded = [0u8; 1];
/// let mut offset = 0;
/// BoolCodec.encode(&value, &mut encoded, &mut offset).unwrap();
/// assert_eq!(offset, 1);
///
/// let mut decode_offset = 0;
/// let decoded: bool = BoolCodec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, value);
/// assert_eq!(decode_offset, 1);
///
/// let size = BoolCodec.measure(&value).unwrap();
/// assert_eq!(size, 1);
///
/// let fixed_size = BoolCodec.measure_fixed();
/// assert_eq!(fixed_size, 1);
/// ```
pub struct BoolCodec;

impl BoolCodec {
    pub const fn new() -> Self {
        Self
    }
}

impl Decoder<'_, '_> for BoolCodec {
    type Decoded = bool;
    fn decode(
        &self,
        encoded: &[u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, error::DecodeError> {
        let byte = U8Codec.decode(encoded, offset)?;
        match byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(error::DecodeError::InvalidData),
        }
    }
}

impl Encoder for BoolCodec {
    type Decoded = bool;
    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), error::EncodeError> {
        U8Codec.encode(
            &match decoded {
                false => 0u8,
                true => 1u8,
            },
            encoded,
            offset,
        )
    }
}

impl FixedMeasurer for BoolCodec {
    fn measure_fixed(&self) -> usize {
        1
    }
}

impl Measurer for BoolCodec {
    type Decoded = bool;
    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, error::EncodeError> {
        Ok(self.measure_fixed())
    }
}

#[cfg(feature = "alloc")]
/// A codec that boxes the decoded value of another codec.
///
/// # Examples
/// ```rust
/// use byten::{BoxCodec, EndianCodec, Decoder, Encoder, FixedMeasurer, Measurer, DecodeError, EncodeError};
///
/// let codec = BoxCodec::new(EndianCodec::<u32>::le());
/// let value: Box<u32> = Box::new(42);
///
/// let mut encoded = [0u8; 4];
/// let mut offset = 0;
/// codec.encode(&value, &mut encoded, &mut offset).unwrap();
/// assert_eq!(offset, 4);
///
/// let mut decode_offset = 0;
/// let decoded: Box<u32> = codec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(*decoded, *value);
/// assert_eq!(decode_offset, 4);
///
/// let size = codec.measure(&value).unwrap();
/// assert_eq!(size, 4);
///
/// let fixed_size = codec.measure_fixed();
/// assert_eq!(fixed_size, 4);
/// ```
pub struct BoxCodec<Codec>(pub Codec);

#[cfg(feature = "alloc")]
impl<Codec> BoxCodec<Codec> {
    pub const fn new(codec: Codec) -> Self {
        Self(codec)
    }
}

#[cfg(feature = "alloc")]
impl<'encoded, 'decoded, Codec, T> Decoder<'encoded, 'decoded> for BoxCodec<Codec>
where
    Codec: Decoder<'encoded, 'decoded, Decoded = T>,
    T: 'decoded,
{
    type Decoded = Box<T>;
    fn decode(
        &self,
        encoded: &'encoded [u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, error::DecodeError> {
        let decoded = self.0.decode(encoded, offset)?;
        Ok(Box::new(decoded))
    }
}

#[cfg(feature = "alloc")]
impl<Codec, T> Encoder for BoxCodec<Codec>
where
    Codec: Encoder<Decoded = T>,
{
    type Decoded = Box<T>;
    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), error::EncodeError> {
        self.0.encode(decoded.as_ref(), encoded, offset)
    }
}

#[cfg(feature = "alloc")]
impl<Codec> FixedMeasurer for BoxCodec<Codec>
where
    Codec: FixedMeasurer,
{
    fn measure_fixed(&self) -> usize {
        self.0.measure_fixed()
    }
}

#[cfg(feature = "alloc")]
impl<Codec> Measurer for BoxCodec<Codec>
where
    Codec: Measurer,
{
    type Decoded = Box<Codec::Decoded>;
    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, error::EncodeError> {
        self.0.measure(decoded.as_ref())
    }
}
