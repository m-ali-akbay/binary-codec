use std::ffi::CStr;

pub struct UTF8Codec<Codec> {
    pub codec: Codec,
}

impl<Codec> UTF8Codec<Codec> {
    pub const fn new(codec: Codec) -> Self {
        Self { codec }
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
        let bytes = self.codec.decode(encoded, offset)?;
        let s = std::str::from_utf8(bytes).map_err(|_| crate::DecodeError::InvalidData)?;
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
        self.codec.encode(decoded.as_bytes(), encoded, offset)
    }
}

impl<Codec> crate::Measurer for UTF8Codec<Codec>
where
    Codec: crate::Measurer<Decoded = [u8]>,
{
    type Decoded = str;

    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, crate::EncodeError> {
        self.codec.measure(decoded.as_bytes())
    }
}

impl<Codec> crate::FixedMeasurer for UTF8Codec<Codec>
where
    Codec: crate::FixedMeasurer,
    Codec: crate::Measurer<Decoded = [u8]>,
{
    fn measure_fixed(&self) -> usize {
        self.codec.measure_fixed()
    }
}

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

pub struct CStrRefCodec;

impl CStrRefCodec {
    pub const fn new() -> Self {
        Self
    }
}

impl<'encoded, 'decoded> crate::Decoder<'encoded, 'decoded> for CStrRefCodec
where
    'encoded: 'decoded,
{
    type Decoded = &'decoded CStr;

    fn decode(
        &self,
        encoded: &'encoded [u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, crate::error::DecodeError> {
        CStrCodec.decode(encoded, offset)
    }
}

impl crate::Encoder for CStrRefCodec {
    type Decoded = CStr;

    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), crate::error::EncodeError> {
        CStrCodec.encode(decoded, encoded, offset)
    }
}

impl crate::Measurer for CStrRefCodec {
    type Decoded = CStr;

    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, crate::error::EncodeError> {
        CStrCodec.measure(decoded)
    }
}
