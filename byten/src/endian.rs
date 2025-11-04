use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub enum Endianness {
    Big,
    Little,
}

pub trait EndianCoded {
    fn length() -> usize;

    fn decode(endianness: Endianness, bytes: &[u8]) -> Self;
    fn encode(&self, endianness: Endianness, bytes: &mut [u8]);
}

pub struct EndianCodec<T> {
    pub endianness: Endianness,
    _marker: PhantomData<T>,
}

impl<T> EndianCodec<T> {
    pub const fn new(endianness: Endianness) -> Self {
        Self {
            endianness,
            _marker: PhantomData,
        }
    }

    pub const fn le() -> Self {
        Self::new(Endianness::Little)
    }

    pub const fn be() -> Self {
        Self::new(Endianness::Big)
    }
}

impl<'decoded, T: EndianCoded + 'decoded> crate::Decoder<'_, 'decoded> for EndianCodec<T> {
    type Decoded = T;

    fn decode(
        &self,
        encoded: &'_ [u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, crate::DecodeError> {
        let length = T::length();
        if *offset + length > encoded.len() {
            return Err(crate::DecodeError::EOF);
        }
        let value = T::decode(self.endianness, &encoded[*offset..*offset + length]);
        *offset += length;
        Ok(value)
    }
}

impl<T: EndianCoded> crate::Encoder for EndianCodec<T> {
    type Decoded = T;

    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), crate::EncodeError> {
        let length = T::length();
        if *offset + length > encoded.len() {
            return Err(crate::EncodeError::BufferTooSmall);
        }
        decoded.encode(self.endianness, &mut encoded[*offset..*offset + length]);
        *offset += length;
        Ok(())
    }
}

impl<T: EndianCoded> crate::FixedMeasurer for EndianCodec<T> {
    fn measure_fixed(&self) -> usize {
        T::length()
    }
}

impl<T: EndianCoded> crate::Measurer for EndianCodec<T> {
    type Decoded = T;

    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, crate::error::EncodeError> {
        Ok(crate::FixedMeasurer::measure_fixed(self))
    }
}

macro_rules! impl_endian_coded {
    ($($ty:ident),* $(,)?) => {
        $(impl EndianCoded for $ty {
            fn length() -> usize {
                (<$ty>::BITS as usize) / 8
            }

            fn decode(endianness: Endianness, bytes: &[u8]) -> Self {
                match endianness {
                    Endianness::Big => <$ty>::from_be_bytes(bytes.try_into().unwrap()),
                    Endianness::Little => <$ty>::from_le_bytes(bytes.try_into().unwrap()),
                }
            }

            fn encode(&self, endianness: Endianness, bytes: &mut [u8]) {
                let byte_array = match endianness {
                    Endianness::Big => self.to_be_bytes(),
                    Endianness::Little => self.to_le_bytes(),
                };
                bytes.copy_from_slice(&byte_array);
            }
        })*
    };
}

impl_endian_coded!(u16, i16, u32, i32, u64, i64, u128, i128,);
