use core::ffi::CStr;
use core::str;

/// A codec for UTF-8 encoded strings.
///
/// The inner codec is used to encode/decode/measure the underlying byte slice.
///
/// # Examples
/// ## Fixed Size UTF-8 String
/// ```rust
/// use byten::{UTF8Codec, Encoder, Decoder, Measurer, FixedMeasurer, BytesCodec, PhantomCodec, EncoderToHeaplessVec as _};
///
/// let s: &str = "Hello, world!";
/// let codec = UTF8Codec::new(BytesCodec::new(PhantomCodec::new(13)));
///
/// let encoded = codec.encode_to_heapless_vec::<20>(s).unwrap();
/// assert_eq!(encoded.len(), 13);
/// assert_eq!(encoded.as_slice(), b"Hello, world!");
///
/// let mut decode_offset = 0;
/// let decoded: &str = codec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, s);
/// assert_eq!(decode_offset, 13);
///
/// let size = codec.measure(s).unwrap();
/// assert_eq!(size, 13);
/// ```
///
/// ## Variable Size UTF-8 String with Length Prefix
/// ```rust
/// use byten::{UTF8Codec, Encoder, Decoder, Measurer, FixedMeasurer, BytesCodec, EndianCodec, EncoderToHeaplessVec as _};
///
/// let s: &str = "Hello, world!";
/// let prefix_codec = EndianCodec::<u16>::le();
/// let bytes_codec = BytesCodec::new(prefix_codec);
/// let codec = UTF8Codec::new(bytes_codec);
///
/// let encoded = codec.encode_to_heapless_vec::<20>(s).unwrap();
/// assert_eq!(encoded.len(), 15);
///
/// let mut decode_offset = 0;
/// let decoded: &str = codec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, s);
/// assert_eq!(decode_offset, 15);
///
/// let size = codec.measure(s).unwrap();
/// assert_eq!(size, 15);
/// ```
pub struct UTF8Codec<Codec>(Codec);

impl<Codec> UTF8Codec<Codec> {
    pub const fn new(codec: Codec) -> Self {
        Self(codec)
    }
}

impl<'encoded, 'decoded, Codec> crate::Decoder<'encoded, 'decoded> for UTF8Codec<Codec>
where
    Codec: crate::Decoder<'encoded, 'decoded, Decoded = &'decoded [u8]>,
    'encoded: 'decoded,
{
    type Decoded = &'decoded str;

    fn decode(
        &self,
        encoded: &'encoded [u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, crate::DecodeError> {
        let bytes = self.0.decode(encoded, offset)?;
        let s = str::from_utf8(bytes).map_err(|_| crate::DecodeError::InvalidData)?;
        Ok(s)
    }
}

impl<Codec> crate::Encoder for UTF8Codec<Codec>
where
    Codec: crate::Encoder<Decoded = [u8]>,
{
    type Decoded = str;

    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), crate::EncodeError> {
        self.0.encode(decoded.as_bytes(), encoded, offset)
    }
}

impl<Codec> crate::Measurer for UTF8Codec<Codec>
where
    Codec: crate::Measurer<Decoded = [u8]>,
{
    type Decoded = str;

    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, crate::EncodeError> {
        self.0.measure(decoded.as_bytes())
    }
}

impl<Codec> crate::FixedMeasurer for UTF8Codec<Codec>
where
    Codec: crate::FixedMeasurer,
    Codec: crate::Measurer<Decoded = [u8]>,
{
    fn measure_fixed(&self) -> usize {
        self.0.measure_fixed()
    }
}

/// A codec for C-style null-terminated strings.
///
/// # Examples
/// ```rust
/// use byten::{CStrCodec, Encoder, Decoder, Measurer, EncoderToHeaplessVec as _};
///
/// let s: &std::ffi::CStr = std::ffi::CStr::from_bytes_with_nul(b"Hello, world!\0").unwrap();
///
/// let codec = CStrCodec::new();
///
/// let encoded = codec.encode_to_heapless_vec::<20>(s).unwrap();
/// assert_eq!(encoded.as_slice(), b"Hello, world!\0");
///
/// let mut decode_offset = 0;
/// let decoded: &std::ffi::CStr = codec.decode(&encoded, &mut decode_offset).unwrap();
///
/// assert_eq!(decoded, s);
/// assert_eq!(decode_offset, encoded.len());
///
/// let size = codec.measure(s).unwrap();
/// assert_eq!(size, encoded.len());
/// ```
pub struct CStrCodec;

impl CStrCodec {
    pub const fn new() -> Self {
        Self
    }
}

impl<'encoded, 'decoded> crate::Decoder<'encoded, 'decoded> for CStrCodec
where
    'encoded: 'decoded,
{
    type Decoded = &'decoded CStr;

    fn decode(
        &self,
        encoded: &'encoded [u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, crate::error::DecodeError> {
        let length = encoded[*offset..]
            .iter()
            .position(|&b| b == 0)
            .ok_or(crate::error::DecodeError::InvalidData)?;
        let cstr = CStr::from_bytes_with_nul(&encoded[*offset..*offset + length + 1])
            .map_err(|_| crate::error::DecodeError::InvalidData)?;
        *offset += length + 1;
        Ok(cstr)
    }
}

impl crate::Encoder for CStrCodec {
    type Decoded = CStr;

    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), crate::error::EncodeError> {
        let bytes = decoded.to_bytes_with_nul();
        let end = *offset + bytes.len();
        if end > encoded.len() {
            return Err(crate::error::EncodeError::BufferTooSmall);
        }
        encoded[*offset..end].copy_from_slice(bytes);
        *offset = end;
        Ok(())
    }
}

impl crate::Measurer for CStrCodec {
    type Decoded = CStr;

    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, crate::error::EncodeError> {
        Ok(decoded.to_bytes_with_nul().len())
    }
}
