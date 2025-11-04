use crate::{Decoder, Encoder, FixedMeasurer, Measurer, error::DecodeError, error::EncodeError};

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
