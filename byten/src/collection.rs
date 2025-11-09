use core::{iter, marker::PhantomData};

use crate::{DecodeError, Decoder, EncodeError, Encoder, Measurer};

pub struct PrefixedCodec<Collection, Length, Item> {
    pub collection: PhantomData<Collection>,
    pub length: Length,
    pub item: Item,
}

impl<Collection, Length, Item> PrefixedCodec<Collection, Length, Item> {
    pub const fn new(length: Length, item: Item) -> Self {
        Self {
            collection: PhantomData,
            length,
            item,
        }
    }
}

impl<'encoded, 'decoded, 'length, Collection, Length, ItemCodec, Item> Decoder<'encoded, 'decoded>
    for PrefixedCodec<Collection, Length, ItemCodec>
where
    Collection: FromIterator<Item> + 'decoded,
    ItemCodec: Decoder<'encoded, 'decoded, Decoded = Item>,
    Length: Decoder<'encoded, 'length>,
    Length::Decoded: TryInto<usize>,
    <Length::Decoded as TryInto<usize>>::Error: Into<DecodeError>,
{
    type Decoded = Collection;

    fn decode(
        &self,
        encoded: &'encoded [u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, DecodeError> {
        let length_decoded = self.length.decode(encoded, offset)?;
        let length: usize = length_decoded.try_into().map_err(Into::into)?;

        let items = iter::from_fn(|| Some(self.item.decode(encoded, offset))).take(length);

        items.collect()
    }
}

impl<Collection, Length, ItemCodec> Measurer for PrefixedCodec<Collection, Length, ItemCodec>
where
    ItemCodec: Measurer,
    for<'decoded> &'decoded Collection: IntoIterator<Item = &'decoded ItemCodec::Decoded>,
    Length: Measurer,
    Length::Decoded: TryFrom<usize>,
    <Length::Decoded as TryFrom<usize>>::Error: Into<EncodeError>,
{
    type Decoded = Collection;

    fn measure(&self, decoded: &Collection) -> Result<usize, EncodeError> {
        let length = decoded.into_iter().count();
        let length: Length::Decoded = length.try_into().map_err(Into::into)?;
        let length_size = self.length.measure(&length)?;
        let items_size =
            decoded
                .into_iter()
                .try_fold(0usize, |acc, item| -> Result<_, EncodeError> {
                    let item_size = self.item.measure(item)?;
                    Ok(acc + item_size)
                })?;
        Ok(length_size + items_size)
    }
}

impl<Collection, Length, ItemCodec> Encoder for PrefixedCodec<Collection, Length, ItemCodec>
where
    ItemCodec: Encoder,
    for<'decoded> &'decoded Collection: IntoIterator<Item = &'decoded ItemCodec::Decoded>,
    Length: Encoder,
    Length::Decoded: TryFrom<usize>,
    <Length::Decoded as TryFrom<usize>>::Error: Into<EncodeError>,
{
    type Decoded = Collection;

    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), EncodeError> {
        let length = decoded.into_iter().count();
        let length: Length::Decoded = length.try_into().map_err(Into::into)?;
        self.length.encode(&length, encoded, offset)?;

        for item in decoded.into_iter() {
            self.item.encode(item, encoded, offset)?;
        }

        Ok(())
    }
}
