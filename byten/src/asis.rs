use crate::{Decoder, Encoder, FixedMeasurer, Measurer, error};

pub struct U8Codec;

impl Decoder<'_, '_> for U8Codec {
    type Decoded = u8;
    fn decode(&self, encoded: &[u8], offset: &mut usize) -> Result<Self::Decoded, error::DecodeError> {
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
    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), error::EncodeError> {
        if *offset + 1 > encoded.len() {
            return Err(error::EncodeError::BufferTooSmall);
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
    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, error::EncodeError> {
        Ok(self.measure_fixed())
    }
}

pub struct I8Codec;

impl Decoder<'_, '_> for I8Codec {
    type Decoded = i8;
    fn decode(&self, encoded: &[u8], offset: &mut usize) -> Result<Self::Decoded, error::DecodeError> {
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
    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), error::EncodeError> {
        if *offset + 1 > encoded.len() {
            return Err(error::EncodeError::BufferTooSmall);
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
    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, error::EncodeError> {
        Ok(self.measure_fixed())
    }
}

pub struct U8ArrayCodec<const N: usize>;

impl<const N: usize> Decoder<'_, '_> for U8ArrayCodec<N> {
    type Decoded = [u8; N];
    fn decode(&self, encoded: &[u8], offset: &mut usize) -> Result<Self::Decoded, error::DecodeError> {
        U8ArrayRefCodec::<N>.decode(encoded, offset).copied()
    }
}

impl<const N: usize> Encoder for U8ArrayCodec<N> {
    type Decoded = [u8; N];
    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), error::EncodeError> {
        U8ArrayRefCodec::<N>.encode(decoded, encoded, offset)
    }
}

impl<const N: usize> FixedMeasurer for U8ArrayCodec<N> {
    fn measure_fixed(&self) -> usize { N }
}

impl<const N: usize> Measurer for U8ArrayCodec<N> {
    type Decoded = [u8; N];
    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, error::EncodeError> {
        Ok(self.measure_fixed())
    }
}

pub struct U8ArrayRefCodec<const N: usize>;

impl<'encoded: 'decoded, 'decoded, const N: usize> Decoder<'encoded, 'decoded> for U8ArrayRefCodec<N> {
    type Decoded = &'decoded [u8; N];
    fn decode(&self, encoded: &'encoded [u8], offset: &mut usize) -> Result<Self::Decoded, error::DecodeError> {
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
    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), error::EncodeError> {
        if *offset + N > encoded.len() {
            return Err(error::EncodeError::BufferTooSmall);
        }
        encoded[*offset..*offset + N].copy_from_slice(decoded);
        *offset += N;
        Ok(())
    }
}

impl<const N: usize> FixedMeasurer for U8ArrayRefCodec<N> {
    fn measure_fixed(&self) -> usize { N }
}

impl<const N: usize> Measurer for U8ArrayRefCodec<N> {
    type Decoded = [u8; N];
    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, error::EncodeError> {
        Ok(self.measure_fixed())
    }
}

pub struct BoolCodec;

impl Decoder<'_, '_> for BoolCodec {
    type Decoded = bool;
    fn decode(&self, encoded: &[u8], offset: &mut usize) -> Result<Self::Decoded, error::DecodeError> {
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
    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), error::EncodeError> {
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
    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, error::EncodeError> {
        Ok(self.measure_fixed())
    }
}

pub struct BoxCodec<Codec>(pub Codec);

impl<'encoded, 'decoded, Codec, T> Decoder<'encoded, 'decoded> for BoxCodec<Codec>
where
    Codec: Decoder<'encoded, 'decoded, Decoded = T>,
    T: 'decoded,
{
    type Decoded = Box<T>;
    fn decode(&self, encoded: &'encoded [u8], offset: &mut usize) -> Result<Self::Decoded, error::DecodeError> {
        let decoded = self.0.decode(encoded, offset)?;
        Ok(Box::new(decoded))
    }
}

impl<Codec, T> Encoder for BoxCodec<Codec>
where
    Codec: Encoder<Decoded = T>,
{
    type Decoded = Box<T>;
    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), error::EncodeError> {
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
    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, error::EncodeError> {
        self.0.measure(decoded.as_ref())
    }
}
