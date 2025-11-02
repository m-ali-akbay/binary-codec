mod endian;
mod array;
mod prelude;
mod util;
mod str;
mod var;

use std::{convert::Infallible, ffi::CStr, marker::PhantomData, mem::MaybeUninit, num::TryFromIntError, ops::Deref};
use thiserror::Error;

#[cfg(feature = "derive")]
pub use byten_derive::{byten, DefaultCodec, Decode, DecodeOwned, Encode, Measure, MeasureFixed};

pub use endian::*;
pub use array::*;
pub use prelude::*;
pub use str::*;
pub use util::*;
pub use var::*;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("End of file reached")]
    EOF,

    #[error("Invalid discriminant")]
    InvalidDiscriminant,

    #[error("Invalid usize")]
    InvalidUSize,

    #[error("Data conversion failure")]
    ConversionFailure,

    #[error("Invalid data")]
    InvalidData,

    #[error("Codec failure")]
    CodecFailure,

    #[error("Bit overflow")]
    BitOverflow,

    #[cfg(feature = "anyhow")]
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl From<Infallible> for DecodeError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<TryFromIntError> for DecodeError {
    fn from(_: TryFromIntError) -> Self {
        DecodeError::CodecFailure
    }
}

#[derive(Error, Debug)]
pub enum EncodeError {
    #[error("Buffer too small")]
    BufferTooSmall,

    #[error("Invalid usize")]
    InvalidUSize,

    #[error("Data conversion failure")]
    CodecFailure,

    #[error("Bit overflow")]
    BitOverflow,

    #[cfg(feature = "anyhow")]
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl From<Infallible> for EncodeError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<TryFromIntError> for EncodeError {
    fn from(_: TryFromIntError) -> Self {
        EncodeError::CodecFailure
    }
}

// codec traits

pub trait Encoder {
    type Decoded: ?Sized;
    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), EncodeError>;
}

impl<T, Ref> Encoder for Ref
where
    T: Encoder,
    Ref: Deref<Target = T>,
{
    type Decoded = T::Decoded;
    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), EncodeError> {
        self.deref().encode(decoded, encoded, offset)
    }
}

pub trait Decoder<'encoded, 'decoded> {
    type Decoded: 'decoded;
    fn decode(&self, encoded: &'encoded [u8], offset: &mut usize) -> Result<Self::Decoded, DecodeError>;
}

impl<'encoded, 'decoded, T, Ref> Decoder<'encoded, 'decoded> for Ref
where
    T: Decoder<'encoded, 'decoded>,
    Ref: Deref<Target = T>,
{
    type Decoded = T::Decoded;
    fn decode(&self, encoded: &'encoded [u8], offset: &mut usize) -> Result<Self::Decoded, DecodeError> {
        self.deref().decode(encoded, offset)
    }
}

pub trait Measurer {
    type Decoded: ?Sized;
    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, EncodeError>;
}

impl<T, Ref> Measurer for Ref
where
    T: Measurer,
    Ref: Deref<Target = T>,
{
    type Decoded = T::Decoded;
    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, EncodeError> {
        self.deref().measure(decoded)
    }
}

pub trait FixedMeasurer: Measurer {
    fn measure_fixed(&self) -> usize;
}

impl<T, Ref> FixedMeasurer for Ref
where
    T: FixedMeasurer,
    Ref: Deref<Target = T>,
{
    fn measure_fixed(&self) -> usize {
        self.deref().measure_fixed()
    }
}

// self coded

pub trait Decode<'encoded> {
    fn decode(encoded: &'encoded [u8], offset: &mut usize) -> Result<Self, DecodeError>
    where
        Self: Sized;
}

pub trait DecodeOwned: for<'encoded> Decode<'encoded> + Sized {
    fn decode_owned(encoded: &[u8], offset: &mut usize) -> Result<Self, DecodeError> {
        Self::decode(encoded, offset)
    }
}

pub trait Encode {
    fn encode(&self, encoded: &mut [u8], offset: &mut usize) -> Result<(), EncodeError>;
}

pub trait Measure {
    fn measure(&self) -> Result<usize, EncodeError>;
}

pub trait MeasureFixed: Measure {
    fn measure_fixed() -> usize;
}

pub struct SelfCoded<T>(PhantomData<T>);

impl<T> SelfCoded<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'encoded, 'decoded, T> Decoder<'encoded, 'decoded> for SelfCoded<T>
where
    T: Decode<'encoded> + 'decoded,
{
    type Decoded = T;
    fn decode(&self, encoded: &'encoded [u8], offset: &mut usize) -> Result<Self::Decoded, DecodeError> {
        T::decode(encoded, offset)
    }
}

impl<T> Encoder for SelfCoded<T>
where
    T: Encode,
{
    type Decoded = T;
    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), EncodeError> {
        decoded.encode(encoded, offset)
    }
}

impl<T> Measurer for SelfCoded<T>
where
    T: Measure,
{
    type Decoded = T;
    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, EncodeError> {
        decoded.measure()
    }
}

impl<T> FixedMeasurer for SelfCoded<T>
where
    T: MeasureFixed,
{
    fn measure_fixed(&self) -> usize {
        T::measure_fixed()
    }
}

// default codec

pub trait DefaultCodec {
    type Codec;
    fn default_codec() -> Self::Codec;
}

impl<T> DefaultCodec for &T
where
    T: DefaultCodec + ?Sized,
{
    type Codec = T::Codec;
    fn default_codec() -> Self::Codec {
        T::default_codec()
    }
}

// very basic implementations

pub struct U8Codec;

impl DefaultCodec for u8 {
    type Codec = U8Codec;
    fn default_codec() -> Self::Codec { U8Codec }
}

impl Decoder<'_, '_> for U8Codec {
    type Decoded = u8;
    fn decode(&self, encoded: &[u8], offset: &mut usize) -> Result<Self::Decoded, DecodeError> {
        if *offset + 1 > encoded.len() {
            return Err(DecodeError::EOF);
        }
        let byte = encoded[*offset];
        *offset += 1;
        Ok(byte)
    }
}

impl Encoder for U8Codec {
    type Decoded = u8;
    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), EncodeError> {
        if *offset + 1 > encoded.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        encoded[*offset] = *decoded;
        *offset += 1;
        Ok(())
    }
}

impl FixedMeasurer for U8Codec {
    fn measure_fixed(&self) -> usize { 1 }
}

impl Measurer for U8Codec {
    type Decoded = u8;
    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, EncodeError> {
        Ok(self.measure_fixed())
    }
}

pub struct I8Codec;

impl DefaultCodec for i8 {
    type Codec = I8Codec;
    fn default_codec() -> Self::Codec { I8Codec }
}

impl Decoder<'_, '_> for I8Codec {
    type Decoded = i8;
    fn decode(&self, encoded: &[u8], offset: &mut usize) -> Result<Self::Decoded, DecodeError> {
        if *offset + 1 > encoded.len() {
            return Err(DecodeError::EOF);
        }
        let byte = encoded[*offset] as i8;
        *offset += 1;
        Ok(byte)
    }
}

impl Encoder for I8Codec {
    type Decoded = i8;
    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), EncodeError> {
        if *offset + 1 > encoded.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        encoded[*offset] = *decoded as u8;
        *offset += 1;
        Ok(())
    }
}

impl FixedMeasurer for I8Codec {
    fn measure_fixed(&self) -> usize { 1 }
}

impl Measurer for I8Codec {
    type Decoded = i8;
    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, EncodeError> {
        Ok(self.measure_fixed())
    }
}

pub struct U8ArrayCodec<const N: usize>;

impl<const N: usize> DefaultCodec for [u8; N] {
    type Codec = U8ArrayCodec<N>;
    fn default_codec() -> Self::Codec { U8ArrayCodec }
}

impl<const N: usize> Decoder<'_, '_> for U8ArrayCodec<N> {
    type Decoded = [u8; N];
    fn decode(&self, encoded: &[u8], offset: &mut usize) -> Result<Self::Decoded, DecodeError> {
        if *offset + N > encoded.len() {
            return Err(DecodeError::EOF);
        }
        let mut array = unsafe { MaybeUninit::<[u8; N]>::uninit().assume_init() };
        array.copy_from_slice(&encoded[*offset..*offset + N]);
        *offset += N;
        Ok(array)
    }
}

impl<const N: usize> Encoder for U8ArrayCodec<N> {
    type Decoded = [u8; N];
    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), EncodeError> {
        if *offset + N > encoded.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        encoded[*offset..*offset + N].copy_from_slice(decoded);
        *offset += N;
        Ok(())
    }
}

impl<const N: usize> FixedMeasurer for U8ArrayCodec<N> {
    fn measure_fixed(&self) -> usize { N }
}

impl<const N: usize> Measurer for U8ArrayCodec<N> {
    type Decoded = [u8; N];
    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, EncodeError> {
        Ok(self.measure_fixed())
    }
}

pub struct BoolCodec;

impl DefaultCodec for bool {
    type Codec = BoolCodec;
    fn default_codec() -> Self::Codec { BoolCodec }
}

impl Decoder<'_, '_> for BoolCodec {
    type Decoded = bool;
    fn decode(&self, encoded: &[u8], offset: &mut usize) -> Result<Self::Decoded, DecodeError> {
        let byte = U8Codec.decode(encoded, offset)?;
        match byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::InvalidData),
        }
    }
}

impl Encoder for BoolCodec {
    type Decoded = bool;
    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), EncodeError> {
        U8Codec.encode(&match decoded {
            false => 0u8,
            true => 1u8,
        }, encoded, offset)
    }
}

impl FixedMeasurer for BoolCodec {
    fn measure_fixed(&self) -> usize { 1 }
}

impl Measurer for BoolCodec {
    type Decoded = bool;
    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, EncodeError> {
        Ok(self.measure_fixed())
    }
}

pub struct BoxCodec<Codec>(pub Codec);

impl<T> DefaultCodec for Box<T>
where
    T: DefaultCodec + ?Sized,
{
    type Codec = BoxCodec<T::Codec>;
    fn default_codec() -> Self::Codec {
        BoxCodec(T::default_codec())
    }
}

impl<'encoded, 'decoded, Codec, T> Decoder<'encoded, 'decoded> for BoxCodec<Codec>
where
    Codec: Decoder<'encoded, 'decoded, Decoded = T>,
    T: 'decoded,
{
    type Decoded = Box<T>;
    fn decode(&self, encoded: &'encoded [u8], offset: &mut usize) -> Result<Self::Decoded, DecodeError> {
        let decoded = self.0.decode(encoded, offset)?;
        Ok(Box::new(decoded))
    }
}

impl<Codec, T> Encoder for BoxCodec<Codec>
where
    Codec: Encoder<Decoded = T>,
{
    type Decoded = Box<T>;
    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), EncodeError> {
        self.0.encode(decoded.as_ref(), encoded, offset)
    }
}

impl<Codec> FixedMeasurer for BoxCodec<Codec>
where
    Codec: FixedMeasurer,
{
    fn measure_fixed(&self) -> usize {
        self.0.measure_fixed()
    }
}

impl<Codec> Measurer for BoxCodec<Codec>
where
    Codec: Measurer,
{
    type Decoded = Box<Codec::Decoded>;
    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, EncodeError> {
        self.0.measure(decoded.as_ref())
    }
}

impl DefaultCodec for CStr {
    type Codec = CStrCodec;
    fn default_codec() -> Self::Codec {
        CStrCodec
    }
}

impl<T> DefaultCodec for Option<T>
where
    T: DefaultCodec,
{
    type Codec = OptionCodec<T::Codec>;
    fn default_codec() -> Self::Codec {
        OptionCodec::new(T::default_codec())
    }
}
