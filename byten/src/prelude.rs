use crate::{DefaultCodec, EncodeError, Encoder, Measurer};

pub trait EncodeToVec {
  fn encode_to_vec(&self) -> Result<Vec<u8>, EncodeError>;
}

impl<T, Codec> EncodeToVec for T
where 
    T: DefaultCodec<Codec=Codec> + ?Sized,
    Codec: Encoder<Decoded=T> + Measurer<Decoded=T>,
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

pub trait EncoderToVec {
    type Decoded: ?Sized;
    fn encode_to_vec(&self, decoded: &Self::Decoded) -> Result<Vec<u8>, EncodeError>;
}

impl<Decoded: ?Sized, C: Encoder<Decoded=Decoded> + Measurer<Decoded=Decoded>> EncoderToVec for C {
    type Decoded = Decoded;
    fn encode_to_vec(&self, decoded: &Self::Decoded) -> Result<Vec<u8>, EncodeError> {
        let size = self.measure(decoded)?;
        let mut vec = vec![0u8; size];
        let mut offset = 0;
        self.encode(decoded, &mut vec, &mut offset)?;
        Ok(vec)
    }
}
