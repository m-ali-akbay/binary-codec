use crate::{Decoder, Encoder, FixedMeasurer, Measurer, error::DecodeError, error::EncodeError};

/// A codec for fixed-size arrays of fixed/dynamic sized elements.
///
/// The size must be a compile-time constant.
/// However, the elements themselves do not need to be of fixed size.
///
/// The item codec is used to encode, decode and measure each element of the array.
///
/// # Examples
/// ```rust
/// use byten::{ArrayCodec, Encoder, FixedMeasurer, Measurer, Decoder, EncodeError, EndianCodec};
///
/// let codec = ArrayCodec::new(EndianCodec::<u32>::le());
/// let array: [u32; 4] = [1, 2, 3, 4];
///
/// let mut encoded = [0u8; 16];
/// let mut offset = 0;
/// codec.encode(&array, &mut encoded, &mut offset).unwrap();
/// assert_eq!(offset, 16);
///
/// let mut decode_offset = 0;
/// let decoded: [u32; 4] = codec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, array);
/// assert_eq!(decode_offset, 16);
///
/// let size = codec.measure(&array).unwrap();
/// assert_eq!(size, 16);
///
/// let fixed_size = codec.measure_fixed();
/// assert_eq!(fixed_size, 16);
/// ```
pub struct ArrayCodec<Item, const N: usize>(pub Item);

impl<Item, const N: usize> ArrayCodec<Item, N> {
    pub const fn new(item: Item) -> Self {
        Self(item)
    }
}

impl<'encoded, 'decoded, Item, const N: usize> Decoder<'encoded, 'decoded> for ArrayCodec<Item, N>
where
    Item: Decoder<'encoded, 'decoded>,
{
    type Decoded = [Item::Decoded; N];

    fn decode(
        &self,
        encoded: &'encoded [u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, DecodeError> {
        let mut array: heapless::Vec<Item::Decoded, N> = heapless::Vec::new();
        for _ in 0..N {
            let item = self.0.decode(encoded, offset)?;
            array
                .push(item)
                .unwrap_or_else(|_| panic!("unexpected heapless vec overflow"));
        }
        let array = array
            .into_array()
            .unwrap_or_else(|_| panic!("unexpected heapless vec underflow"));
        Ok(array)
    }
}

impl<Item, const N: usize> Encoder for ArrayCodec<Item, N>
where
    Item: Encoder,
    Item::Decoded: Sized,
{
    type Decoded = [Item::Decoded; N];

    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), EncodeError> {
        for item in decoded.iter() {
            self.0.encode(item, encoded, offset)?;
        }
        Ok(())
    }
}

impl<Item, const N: usize> FixedMeasurer for ArrayCodec<Item, N>
where
    Item: FixedMeasurer,
    Item::Decoded: Sized,
{
    fn measure_fixed(&self) -> usize {
        N * self.0.measure_fixed()
    }
}

impl<Item, const N: usize> Measurer for ArrayCodec<Item, N>
where
    Item: Measurer,
    Item::Decoded: Sized,
{
    type Decoded = [Item::Decoded; N];

    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, EncodeError> {
        let mut total = 0;
        for item in decoded.iter() {
            total += self.0.measure(item)?;
        }
        Ok(total)
    }
}
