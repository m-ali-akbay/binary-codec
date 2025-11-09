#[cfg(feature = "alloc")]
use core::option::Option;

use crate::BoolCodec;

/// A codec that draws all remaining bytes from the input during decoding,
/// and writes all bytes during encoding.
///
/// This codec is useful for handling trailing data of unknown/unbounded length.
///
/// # Examples
/// ```rust
/// use byten::{RemainingCodec, Encoder, Decoder, Measurer, EncoderToVec as _};
///
/// let codec = RemainingCodec::new();
/// let data: &[u8] = b"Hello, world!";
///
/// let encoded = codec.encode_to_heapless_vec::<20>(data).unwrap();
/// assert_eq!(encoded.as_slice(), b"Hello, world!");
///
/// let mut decode_offset = 0;
/// let decoded: &[u8] = codec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, data);
/// assert_eq!(decode_offset, encoded.len());
/// ```
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

    fn decode(
        &self,
        encoded: &'encoded [u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, crate::error::DecodeError> {
        if *offset > encoded.len() {
            return Err(crate::error::DecodeError::InvalidData);
        }
        let remaining = &encoded[*offset..];
        *offset = encoded.len();
        Ok(remaining)
    }
}

impl crate::Encoder for RemainingCodec {
    type Decoded = [u8];

    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), crate::error::EncodeError> {
        let end = *offset + decoded.len();
        if end > encoded.len() {
            return Err(crate::error::EncodeError::BufferTooSmall);
        }
        encoded[*offset..end].copy_from_slice(decoded);
        *offset = end;
        Ok(())
    }
}

impl crate::Measurer for RemainingCodec {
    type Decoded = [u8];

    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, crate::error::EncodeError> {
        Ok(decoded.len())
    }
}

/// A codec for optional values.
/// The presence of a value is indicated by a preceding boolean flag.
/// If the flag is true, the value is absent (None).
/// If the flag is false, the value is present (Some).
///
/// This coding is not fixed-size, as the size depends on whether the value is present or not.
/// Unlike how Rust stores the Option type in the memory (fixed size and with compacted discriminant for some optimized types),
/// this codec always uses one byte for the presence flag, followed by the encoded value if present
///
/// # Examples
/// ```rust
/// use byten::{OptionCodec, Encoder, Decoder, Measurer, EncoderToVec as _};
///
/// let item_codec = byten::EndianCodec::<u32>::le();
/// let codec = OptionCodec::new(item_codec);
/// let some_value = Some(0x12345678u32);
/// let none_value: Option<u32> = None;
///
/// let encoded_some = codec.encode_to_heapless_vec::<10>(&some_value).unwrap();
/// let encoded_none = codec.encode_to_heapless_vec::<10>(&none_value).unwrap();
///
/// let mut decode_offset = 0;
/// let decoded_some: Option<u32> = codec.decode(&encoded_some, &mut decode_offset).unwrap();
/// assert_eq!(decoded_some, some_value);
///
/// decode_offset = 0;
/// let decoded_none: Option<u32> = codec.decode(&encoded_none, &mut decode_offset).unwrap();
/// assert_eq!(decoded_none, none_value);
///
/// let some_size = codec.measure(&some_value).unwrap();
/// let none_size = codec.measure(&none_value).unwrap();
/// assert!(some_size > none_size);
/// assert_eq!(none_size, 1);
/// ```
pub struct OptionCodec<Item>(Item);

impl<Item> OptionCodec<Item> {
    pub fn new(item: Item) -> Self {
        Self(item)
    }
}

impl<'encoded, 'decoded, Item> crate::Decoder<'encoded, 'decoded> for OptionCodec<Item>
where
    Item: crate::Decoder<'encoded, 'decoded>,
{
    type Decoded = Option<Item::Decoded>;

    fn decode(
        &self,
        encoded: &'encoded [u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, crate::error::DecodeError> {
        let flag = BoolCodec.decode(encoded, offset)?;
        if flag {
            Ok(Option::None)
        } else {
            let item = self.0.decode(encoded, offset)?;
            Ok(Option::Some(item))
        }
    }
}

impl<Item> crate::Encoder for OptionCodec<Item>
where
    Item: crate::Encoder,
    Item::Decoded: Sized,
{
    type Decoded = Option<Item::Decoded>;

    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), crate::error::EncodeError> {
        match decoded {
            Option::None => {
                BoolCodec.encode(&true, encoded, offset)?;
                Ok(())
            }
            Option::Some(item) => {
                BoolCodec.encode(&false, encoded, offset)?;
                self.0.encode(item, encoded, offset)
            }
        }
    }
}

impl<Item> crate::Measurer for OptionCodec<Item>
where
    Item: crate::Measurer,
    Item::Decoded: Sized,
{
    type Decoded = Option<Item::Decoded>;

    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, crate::error::EncodeError> {
        Ok(match decoded {
            Option::None => BoolCodec.measure(&true)?,
            Option::Some(item) => BoolCodec.measure(&false)? + self.0.measure(item)?,
        })
    }
}

/// A codec for byte slices with a length prefix.
/// The length of the byte slice is encoded/decoded using the provided length codec.
///
/// # Examples
/// ```rust
/// use byten::{BytesCodec, Encoder, Decoder, Measurer, EncoderToVec as _};
///
/// let length_codec = byten::EndianCodec::<u16>::le();
/// let codec = BytesCodec::new(length_codec);
/// let data: &[u8] = b"Hello, world!";
///
/// let encoded = codec.encode_to_heapless_vec::<20>(data).unwrap();
/// assert_eq!(encoded.len(), 2 + data.len());
///
/// let mut decode_offset = 0;
/// let decoded: &[u8] = codec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, data);
/// assert_eq!(decode_offset, encoded.len());
///
/// let size = codec.measure(data).unwrap();
/// assert_eq!(size, 2 + data.len());
/// ```
pub struct BytesCodec<Length>(Length);

impl<Length> BytesCodec<Length> {
    pub const fn new(length: Length) -> Self {
        Self(length)
    }
}

impl<'encoded, 'decoded, 'length, Length> crate::Decoder<'encoded, 'decoded> for BytesCodec<Length>
where
    Length: crate::Decoder<'encoded, 'length>,
    Length::Decoded: TryInto<usize>,
    <Length::Decoded as TryInto<usize>>::Error: Into<crate::error::DecodeError>,
    'encoded: 'decoded,
{
    type Decoded = &'decoded [u8];

    fn decode(
        &self,
        encoded: &'encoded [u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, crate::error::DecodeError> {
        let size = self
            .0
            .decode(encoded, offset)?
            .try_into()
            .map_err(Into::into)?;
        if *offset + size > encoded.len() {
            return Err(crate::error::DecodeError::InvalidData);
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
    <Length::Decoded as TryFrom<usize>>::Error: Into<crate::error::EncodeError>,
{
    type Decoded = [u8];

    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), crate::error::EncodeError> {
        let size = decoded.len();
        self.0
            .encode(&size.try_into().map_err(Into::into)?, encoded, offset)?;
        let end = *offset + size;
        if end > encoded.len() {
            return Err(crate::error::EncodeError::BufferTooSmall);
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
    <Length::Decoded as TryFrom<usize>>::Error: Into<crate::error::EncodeError>,
{
    type Decoded = [u8];

    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, crate::error::EncodeError> {
        let size = decoded.len();
        let size_measure = self.0.measure(&size.try_into().map_err(Into::into)?)?;
        Ok(size_measure + size)
    }
}
