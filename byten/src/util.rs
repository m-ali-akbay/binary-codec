use core::borrow;
use std::{borrow::Borrow as _, ops::Deref};

pub struct OwnedCodec<Codec> {
    pub codec: Codec,
}

impl<Codec> OwnedCodec<Codec> {
    pub const fn new(codec: Codec) -> Self {
        Self { codec }
    }
}

impl<'encoded, 'decoded, Codec> crate::Decoder<'encoded, 'decoded> for OwnedCodec<Codec>
where
    Codec: crate::Decoder<'encoded, 'decoded>,
    Codec::Decoded: Deref,
    <Codec::Decoded as Deref>::Target: ToOwned,
{
    type Decoded = <<Codec::Decoded as Deref>::Target as ToOwned>::Owned;
    fn decode(&self, encoded: &'encoded [u8], offset: &mut usize) -> Result<Self::Decoded, crate::DecodeError> {
        let borrowed = self.codec.decode(encoded, offset)?;
        Ok(borrowed.deref().to_owned())
    }
}

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
