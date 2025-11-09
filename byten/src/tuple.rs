/// Codec for the unit type `()`.
/// The unit codec encodes and decodes the unit type without consuming any bytes.
///
/// # Examples
/// ```rust
/// use byten::{UnitCodec, Decoder, Encoder, Measurer, FixedMeasurer};
///
/// let codec = UnitCodec::new();
///
/// let mut encoded = [0u8; 0];
/// let mut offset = 0;
/// codec.encode(&(), &mut encoded, &mut offset).unwrap();
/// assert_eq!(offset, 0);
///
/// let mut decode_offset = 0;
/// let decoded: () = codec.decode(&encoded, &mut decode_offset).unwrap();
/// assert_eq!(decoded, ());
/// assert_eq!(decode_offset, 0);
///
/// let size = codec.measure(&()).unwrap();
/// assert_eq!(size, 0);
///
/// let fixed_size = codec.measure_fixed();
/// assert_eq!(fixed_size, 0);
/// ```
pub struct UnitCodec;

impl UnitCodec {
    pub const fn new() -> Self {
        Self
    }
}

impl crate::Decoder<'_, '_> for UnitCodec {
    type Decoded = ();

    fn decode(
        &self,
        _encoded: &'_ [u8],
        _offset: &mut usize,
    ) -> Result<Self::Decoded, crate::error::DecodeError> {
        Ok(())
    }
}

impl crate::Encoder for UnitCodec {
    type Decoded = ();

    fn encode(
        &self,
        _decoded: &Self::Decoded,
        _encoded: &mut [u8],
        _offset: &mut usize,
    ) -> Result<(), crate::error::EncodeError> {
        Ok(())
    }
}

impl crate::Measurer for UnitCodec {
    type Decoded = ();

    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, crate::error::EncodeError> {
        Ok(0)
    }
}

impl crate::FixedMeasurer for UnitCodec {
    fn measure_fixed(&self) -> usize {
        0
    }
}

macro_rules! tuple_codecs {
    ($name:ident, ($($idx:tt),*), ($($codec:ident),*), ($($codec_var:ident),*), ($($item:ident),*)) => {
        pub struct $name<$($codec),*>($(pub $codec),*);

        impl <$($codec),*> $name<$($codec),*> {
            pub const fn new($($codec_var: $codec),*) -> Self {
                Self( $($codec_var),* )
            }
        }

        impl<'encoded, 'decoded, $($codec),*> crate::Decoder<'encoded, 'decoded> for $name<$($codec),*>
        where
            $($codec: crate::Decoder<'encoded, 'decoded>,)*
        {
            type Decoded = ($($codec::Decoded),*,);

            fn decode(
                &self,
                encoded: &'encoded [u8],
                offset: &mut usize,
            ) -> Result<Self::Decoded, crate::error::DecodeError> {
                let decoded = ( $( self.$idx.decode(encoded, offset)? ),* , );
                Ok(decoded)
            }
        }

        impl<$($codec),*, $($item: Sized),*> crate::Encoder for $name<$($codec),*>
        where
            $(
                $codec: crate::Encoder<Decoded = $item>,
            )*
        {
            type Decoded = ($($item),*,);

            fn encode(
                &self,
                decoded: &Self::Decoded,
                encoded: &mut [u8],
                offset: &mut usize,
            ) -> Result<(), crate::error::EncodeError> {
                $( self.$idx.encode(&decoded.$idx, encoded, offset)?; )*
                Ok(())
            }
        }

        impl<$($codec),*, $($item: Sized),*> crate::Measurer for $name<$($codec),*>
        where
            $($codec: crate::Measurer<Decoded = $item>,)*
        {
            type Decoded = ($($item),*,);

            fn measure(&self, decoded: &Self::Decoded) -> Result<usize, crate::error::EncodeError> {
                let mut size: usize = 0;
                $( size += self.$idx.measure(&decoded.$idx)?; )*
                Ok(size)
            }
        }

        impl<'decoded, $($codec),*, $($item: 'decoded + Sized),*> crate::FixedMeasurer for $name<$($codec),*>
        where
            $($codec: crate::FixedMeasurer<Decoded = &'decoded $item>,)*
        {
            fn measure_fixed(&self) -> usize {
                0usize $( + self.$idx.measure_fixed() )*
            }
        }
    };
}

pub type Tuple0Codec = UnitCodec;

tuple_codecs!(Tuple1Codec, (0), (Codec1), (codec1), (Item1));
tuple_codecs!(
    Tuple2Codec,
    (0, 1),
    (Codec1, Codec2),
    (codec1, codec2),
    (Item1, Item2)
);
tuple_codecs!(
    Tuple3Codec,
    (0, 1, 2),
    (Codec1, Codec2, Codec3),
    (codec1, codec2, codec3),
    (Item1, Item2, Item3)
);
tuple_codecs!(
    Tuple4Codec,
    (0, 1, 2, 3),
    (Codec1, Codec2, Codec3, Codec4),
    (codec1, codec2, codec3, codec4),
    (Item1, Item2, Item3, Item4)
);
tuple_codecs!(
    Tuple5Codec,
    (0, 1, 2, 3, 4),
    (Codec1, Codec2, Codec3, Codec4, Codec5),
    (codec1, codec2, codec3, codec4, codec5),
    (Item1, Item2, Item3, Item4, Item5)
);
tuple_codecs!(
    Tuple6Codec,
    (0, 1, 2, 3, 4, 5),
    (Codec1, Codec2, Codec3, Codec4, Codec5, Codec6),
    (codec1, codec2, codec3, codec4, codec5, codec6),
    (Item1, Item2, Item3, Item4, Item5, Item6)
);
tuple_codecs!(
    Tuple7Codec,
    (0, 1, 2, 3, 4, 5, 6),
    (Codec1, Codec2, Codec3, Codec4, Codec5, Codec6, Codec7),
    (codec1, codec2, codec3, codec4, codec5, codec6, codec7),
    (Item1, Item2, Item3, Item4, Item5, Item6, Item7)
);
