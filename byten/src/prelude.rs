use crate::{Decoder, DefaultCodec, Encoder, Measurer, error::DecodeError, error::EncodeError};
#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

pub trait DecodeDefault<'encoded>: Sized {
    fn decode(
        encoded: &'encoded [u8],
        offset: &mut usize,
    ) -> Result<Self, crate::error::DecodeError>;
}

impl<'encoded, 'decoded, T> DecodeDefault<'encoded> for T
where
    T: DefaultCodec + 'decoded,
    T::Codec: Decoder<'encoded, 'decoded, Decoded = T>,
{
    fn decode(encoded: &'encoded [u8], offset: &mut usize) -> Result<Self, DecodeError> {
        let codec = T::default_codec();
        codec.decode(encoded, offset)
    }
}

pub trait EncodeDefault {
    fn encode(&self, encoded: &mut [u8], offset: &mut usize) -> Result<(), EncodeError>;
}

impl<T, Codec> EncodeDefault for T
where
    T: DefaultCodec<Codec = Codec> + ?Sized,
    Codec: Encoder<Decoded = T>,
{
    fn encode(&self, encoded: &mut [u8], offset: &mut usize) -> Result<(), EncodeError> {
        let codec = T::default_codec();
        codec.encode(self, encoded, offset)
    }
}

#[cfg(feature = "alloc")]
pub trait EncodeToVec {
    fn encode_to_vec(&self) -> Result<Vec<u8>, EncodeError>;
}

#[cfg(feature = "alloc")]
impl<T, Codec> EncodeToVec for T
where
    T: DefaultCodec<Codec = Codec> + ?Sized,
    Codec: Encoder<Decoded = T> + Measurer<Decoded = T>,
{
    fn encode_to_vec(&self) -> Result<Vec<u8>, EncodeError> {
        let codec = Self::default_codec();
        let size = codec.measure(self)?;
        let mut vec = vec![0u8; size];
        let mut offset = 0;
        codec.encode(self, &mut vec, &mut offset)?;
        Ok(vec)
    }
}

pub trait EncodeToHeaplessVec {
    fn encode_to_heapless_vec<const N: usize>(
        &self,
    ) -> Result<heapless::Vec<u8, N>, EncodeError>;
}

impl<T, Codec> EncodeToHeaplessVec for T
where
    T: DefaultCodec<Codec = Codec> + ?Sized,
    Codec: Encoder<Decoded = T> + Measurer<Decoded = T>,
{
    fn encode_to_heapless_vec<const N: usize>(
        &self,
    ) -> Result<heapless::Vec<u8, N>, EncodeError> {
        let codec = Self::default_codec();
        let size = codec.measure(self)?;
        if size > N {
            return Err(EncodeError::BufferTooSmall);
        }
        let mut vec = heapless::Vec::<u8, N>::new();
        vec.resize_default(size).map_err(|_| EncodeError::BufferTooSmall)?;
        let mut offset = 0;
        codec.encode(self, &mut vec, &mut offset)?;
        Ok(vec)
    }
}

#[cfg(feature = "alloc")]
pub trait EncoderToVec {
    type Decoded: ?Sized;
    fn encode_to_vec(&self, decoded: &Self::Decoded) -> Result<Vec<u8>, EncodeError>;
}

#[cfg(feature = "alloc")]
impl<Decoded: ?Sized, C: Encoder<Decoded = Decoded> + Measurer<Decoded = Decoded>> EncoderToVec
    for C
{
    type Decoded = Decoded;
    fn encode_to_vec(&self, decoded: &Self::Decoded) -> Result<Vec<u8>, crate::error::EncodeError> {
        let size = self.measure(decoded)?;
        let mut vec = vec![0u8; size];
        let mut offset = 0;
        self.encode(decoded, &mut vec, &mut offset)?;
        Ok(vec)
    }
}

pub trait EncoderToHeaplessVec {
    type Decoded: ?Sized;
    fn encode_to_heapless_vec<const N: usize>(
        &self,
        decoded: &Self::Decoded,
    ) -> Result<heapless::Vec<u8, N>, EncodeError>;
}

impl<Decoded: ?Sized, C: Encoder<Decoded = Decoded> + Measurer<Decoded = Decoded>>
    EncoderToHeaplessVec for C
{
    type Decoded = Decoded;
    fn encode_to_heapless_vec<const N: usize>(
        &self,
        decoded: &Self::Decoded,
    ) -> Result<heapless::Vec<u8, N>, crate::error::EncodeError> {
        let size = self.measure(decoded)?;
        if size > N {
            return Err(crate::error::EncodeError::BufferTooSmall);
        }
        let mut vec = heapless::Vec::<u8, N>::new();
        vec.resize_default(size).map_err(|_| crate::error::EncodeError::BufferTooSmall)?;
        let mut offset = 0;
        self.encode(decoded, &mut vec, &mut offset)?;
        Ok(vec)
    }
}
