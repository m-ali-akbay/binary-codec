#[cfg(feature = "alloc")]
use alloc::borrow::ToOwned;
use core::{borrow, borrow::Borrow as _, ops::Deref};

#[cfg(feature = "alloc")]
/// A codec that decodes to owned data by first decoding to borrowed data
/// and then cloning it.
///
/// # Examples
/// ## Fixed Size UTF-8 String
/// ```rust
/// use byten::{OwnedCodec, UTF8Codec, Encoder, Decoder, Measurer, FixedMeasurer, BytesCodec, PhantomCodec, EncoderToVec as _};
///
/// let s: String = "Hello, world!".to_owned();
/// let str_codec = UTF8Codec::new(BytesCodec::new(PhantomCodec::new(13)));
/// let codec = OwnedCodec::new(str_codec);
///
/// let encoded = codec.encode_to_vec(&s).unwrap();
/// assert_eq!(encoded.len(), 13);
/// assert_eq!(encoded, b"Hello, world!");
///
/// let mut decode_offset = 0;
/// let decoded: String = codec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, s);
/// assert_eq!(decode_offset, 13);
///
/// let size = codec.measure(&s).unwrap();
/// assert_eq!(size, 13);
/// ```
pub struct OwnedCodec<Codec> {
    pub codec: Codec,
}

#[cfg(feature = "alloc")]
impl<Codec> OwnedCodec<Codec> {
    pub const fn new(codec: Codec) -> Self {
        Self { codec }
    }
}

#[cfg(feature = "alloc")]
impl<'encoded, 'decoded, Codec> crate::Decoder<'encoded, 'decoded> for OwnedCodec<Codec>
where
    Codec: crate::Decoder<'encoded, 'decoded>,
    Codec::Decoded: Deref,
    <Codec::Decoded as Deref>::Target: ToOwned,
{
    type Decoded = <<Codec::Decoded as Deref>::Target as ToOwned>::Owned;
    fn decode(
        &self,
        encoded: &'encoded [u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, crate::DecodeError> {
        let borrowed = self.codec.decode(encoded, offset)?;
        Ok(borrowed.deref().to_owned())
    }
}

#[cfg(feature = "alloc")]
impl<Codec> crate::Encoder for OwnedCodec<Codec>
where
    Codec: crate::Encoder,
    Codec::Decoded: ToOwned,
    <Codec::Decoded as ToOwned>::Owned: borrow::Borrow<Codec::Decoded>,
{
    type Decoded = <Codec::Decoded as ToOwned>::Owned;
    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), crate::EncodeError> {
        self.codec.encode(decoded.borrow(), encoded, offset)
    }
}

#[cfg(feature = "alloc")]
impl<Codec> crate::Measurer for OwnedCodec<Codec>
where
    Codec: crate::Measurer,
    Codec::Decoded: ToOwned,
    <Codec::Decoded as ToOwned>::Owned: borrow::Borrow<Codec::Decoded>,
{
    type Decoded = <Codec::Decoded as ToOwned>::Owned;
    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, crate::EncodeError> {
        self.codec.measure(decoded.borrow())
    }
}

#[cfg(feature = "alloc")]
impl<Codec> crate::FixedMeasurer for OwnedCodec<Codec>
where
    Codec: crate::FixedMeasurer,
    Codec::Decoded: ToOwned,
    <Codec::Decoded as ToOwned>::Owned: borrow::Borrow<Codec::Decoded>,
{
    fn measure_fixed(&self) -> usize {
        self.codec.measure_fixed()
    }
}

/// A trait for codecs that always encode and decode to a constant value.
pub trait ConstantCoder {
    type Decoded;
    fn constant(&self) -> &Self::Decoded;
}

/// A codec that always encodes and decodes to a constant value.
/// It does not read or write any bytes.
/// And it measures to fixed zero bytes.
///
/// This is useful as a placeholder codec for types that do not require any storage.
/// Or in place of length-prefixed codecs when the length is known to be a constant, and
/// no length prefix is needed.
///
/// # Examples
/// ```rust
/// use byten::{PhantomCodec, Encoder, Decoder, Measurer, FixedMeasurer, EncoderToVec as _};
///
/// let codec = PhantomCodec::new(42u32);
///
/// let encoded = codec.encode_to_vec(&42u32).unwrap();
/// assert_eq!(encoded.len(), 0);
/// assert_eq!(encoded, vec![]);
///
/// let mut decode_offset = 0;
/// let decoded: u32 = codec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, 42u32);
/// assert_eq!(decode_offset, 0);
///
/// let size = codec.measure(&42u32).unwrap();
/// assert_eq!(size, 0);
///
/// let fixed_size = codec.measure_fixed();
/// assert_eq!(fixed_size, 0);
/// ```
pub struct PhantomCodec<T>(pub T);

impl<T> PhantomCodec<T>
where
    T: Clone,
{
    pub const fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> ConstantCoder for PhantomCodec<T> {
    type Decoded = T;
    fn constant(&self) -> &Self::Decoded {
        &self.0
    }
}

impl<'encoded, 'decoded, T> crate::Decoder<'encoded, 'decoded> for PhantomCodec<T>
where
    T: Clone + 'decoded,
{
    type Decoded = T;
    fn decode(
        &self,
        _encoded: &'encoded [u8],
        _offset: &mut usize,
    ) -> Result<Self::Decoded, crate::DecodeError> {
        Ok(self.0.clone())
    }
}

impl<T> crate::Encoder for PhantomCodec<T>
where
    T: Clone + PartialEq,
{
    type Decoded = T;
    fn encode(
        &self,
        _decoded: &Self::Decoded,
        _encoded: &mut [u8],
        _offset: &mut usize,
    ) -> Result<(), crate::EncodeError> {
        if _decoded.ne(&self.0) {
            return Err(crate::EncodeError::InvalidData);
        }
        Ok(())
    }
}

impl<T> crate::Measurer for PhantomCodec<T>
where
    T: Clone + PartialEq,
{
    type Decoded = T;
    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, crate::EncodeError> {
        if _decoded.ne(&self.0) {
            return Err(crate::EncodeError::InvalidData);
        }
        Ok(0)
    }
}

impl<T> crate::FixedMeasurer for PhantomCodec<T>
where
    T: Clone + PartialEq,
{
    fn measure_fixed(&self) -> usize {
        0
    }
}
