use std::marker::PhantomData;

use crate::{Decoder, Encoder, FixedMeasurer, Measurer, error};

/// A trait for types that can decode themselves from a byte slice.
/// The lifetime 'encoded refers to the lifetime of the byte slice being decoded.
/// 
/// All the behavior should be same as implementing a `Decoder` for Self.
/// 
/// # Examples
/// ## Decoding owned data
/// ```rust
/// use byten::{Decode, DecodeError};
/// 
/// struct MyType {
///     value: u32,
/// }
/// 
/// impl Decode<'_> for MyType {
///     fn decode(encoded: &[u8], offset: &mut usize) -> Result<Self, DecodeError> {
///         if *offset + 4 > encoded.len() {
///             return Err(DecodeError::EOF);
///         }
///         let value = u32::from_le_bytes(encoded[*offset..*offset + 4].try_into().expect("length is already checked"));
///         *offset += 4;
///         Ok(MyType { value })
///     }
/// }
/// 
/// let data = vec![0x78, 0x56, 0x34, 0x12];
/// let mut offset = 0;
/// let my_type = MyType::decode(&data, &mut offset).unwrap();
/// assert_eq!(my_type.value, 0x12345678);
/// assert_eq!(offset, 4);
/// ```
/// 
/// ## Decoding borrowed data
/// ```rust
/// use byten::{Decode, DecodeError};
/// 
/// struct MyType<'a> {
///     value: &'a [u8; 4],
/// }
/// 
/// impl<'encoded: 'decoded, 'decoded> Decode<'encoded> for MyType<'decoded> {
///     fn decode(encoded: &'encoded [u8], offset: &mut usize) -> Result<Self, DecodeError> {
///         if *offset + 4 > encoded.len() {
///             return Err(DecodeError::EOF);
///         }
///         let value = encoded[*offset..*offset + 4].try_into().expect("length is already checked");
///         *offset += 4;
///         Ok(MyType { value })
///     }
/// }
/// 
/// let data = vec![0x78, 0x56, 0x34, 0x12];
/// let mut offset = 0;
/// let my_type = MyType::decode(&data, &mut offset).unwrap();
/// assert_eq!(my_type.value, &[0x78, 0x56, 0x34, 0x12]);
/// assert_eq!(offset, 4);
pub trait Decode<'encoded> {
    fn decode(encoded: &'encoded [u8], offset: &mut usize) -> Result<Self, error::DecodeError>
    where
        Self: Sized;
}

/// A trait for types that can decode themselves from a byte slice,
/// returning an owned instance.
///
/// The trait can be implemented for types that implement `Decode` for all lifetimes.
///
/// If the type does not implement `Decode` for all lifetimes, this trait can not be implemented.
///
/// # Examples
/// ## Decoding a simple struct
/// ```rust
/// use byten::{Decode, DecodeOwned, DecodeError};
/// 
/// struct MyType {
///     value: u32,
/// }
/// 
/// impl Decode<'_> for MyType {
///     fn decode(encoded: &[u8], offset: &mut usize) -> Result<Self, DecodeError> {
///         if *offset + 4 > encoded.len() {
///             return Err(DecodeError::EOF);
///         }
///         let value = u32::from_le_bytes(encoded[*offset..*offset + 4].try_into().expect("length is already checked"));
///         *offset += 4;
///         Ok(MyType { value })
///     }
/// }
/// 
/// impl DecodeOwned for MyType {}
/// 
/// let data = vec![0x78, 0x56, 0x34, 0x12];
/// let mut offset = 0;
/// let my_type = MyType::decode_owned(&data, &mut offset).unwrap();
/// assert_eq!(my_type.value, 0x12345678);
/// assert_eq!(offset, 4);
/// ```
pub trait DecodeOwned: for<'encoded> Decode<'encoded> + Sized {
    fn decode_owned(encoded: &[u8], offset: &mut usize) -> Result<Self, error::DecodeError> {
        Self::decode(encoded, offset)
    }
}

/// A trait for types that can encode themselves into a byte slice.
/// 
/// All the behavior should be same as implementing an `Encoder` for Self.
/// 
/// # Examples
/// ## Encoding simple struct
/// ```rust
/// use byten::{Encode, EncodeError};
/// 
/// struct MyType {
///     value: u32,
/// }
/// 
/// impl Encode for MyType {
///     fn encode(&self, encoded: &mut [u8], offset: &mut usize) -> Result<(), EncodeError> {
///         if *offset + 4 > encoded.len() {
///             return Err(EncodeError::BufferTooSmall);
///         }
///         encoded[*offset..*offset + 4].copy_from_slice(&self.value.to_le_bytes());
///         *offset += 4;
///         Ok(())
///     }
/// }
/// 
/// let my_type = MyType { value: 0x12345678 };
/// let mut data = vec![0u8; 4];
/// let mut offset = 0;
/// my_type.encode(&mut data, &mut offset).unwrap();
/// assert_eq!(data, vec![0x78, 0x56, 0x34, 0x12]);
/// assert_eq!(offset, 4);
/// ```
pub trait Encode {
    fn encode(&self, encoded: &mut [u8], offset: &mut usize) -> Result<(), error::EncodeError>;
}

/// A trait for types that can measure the size of their encoded representation.
/// 
/// # Examples
/// ## Measuring simple struct
/// ```rust
/// use byten::{Measure, EncodeError};
/// 
/// struct MyType {
///     value: u32,
/// }
/// 
/// impl Measure for MyType {
///     fn measure(&self) -> Result<usize, EncodeError> {
///         Ok(4)
///     }
/// }
/// 
/// let my_type = MyType { value: 0x12345678 };
/// let size = my_type.measure().unwrap();
/// assert_eq!(size, 4);
/// ```
pub trait Measure {
    fn measure(&self) -> Result<usize, error::EncodeError>;
}

/// A trait for types that have a fixed size when encoded.
/// 
/// The size can be determined without needing an instance of the type.
/// 
/// The `MeasureFixed` trait extends the `Measure` trait.
/// Thus, any type that implements `MeasureFixed` must also implement `Measure`.
/// 
/// # Examples
/// ## Measuring fixed size struct
/// ```rust
/// use byten::{MeasureFixed, Measure, EncodeError};
/// 
/// struct MyType {
///     value: u32,
/// }
/// 
/// impl MeasureFixed for MyType {
///     fn measure_fixed() -> usize {
///         4
///     }
/// }
/// 
/// impl Measure for MyType {
///     fn measure(&self) -> Result<usize, EncodeError> {
///         Ok(4)
///     }
/// }
/// 
/// let size = MyType::measure_fixed();
/// assert_eq!(size, 4);
/// ```
pub trait MeasureFixed: Measure {
    fn measure_fixed() -> usize;
}

/// A wrapper codec that re-uses the Decode, Encode, Measure, and MeasureFixed traits of the decoded type.
///
/// # Examples
/// ```rust
/// use byten::{
///     SelfCoded, Decode, Encode, Measure, MeasureFixed,
///     Decoder, Encoder, Measurer, FixedMeasurer,
///     DecodeError, EncodeError,
///     EndianCodec,
/// };
/// 
/// struct MyType {
///     value: u32,
/// }
/// 
/// impl MyType {
///     fn new(value: u32) -> Self {
///         Self { value }
///     }
/// }
/// 
/// impl<'encoded> Decode<'encoded> for MyType {
///     fn decode(encoded: &'encoded [u8], offset: &mut usize) -> Result<Self, DecodeError> {
///         Ok(MyType {
///             value: EndianCodec::<u32>::le().decode(encoded, offset)?
///         })
///     }
/// }
///
/// impl Encode for MyType {
///     fn encode(&self, encoded: &mut [u8], offset: &mut usize) -> Result<(), EncodeError> {
///         EndianCodec::<u32>::le().encode(&self.value, encoded, offset)
///     }
/// }
/// 
/// impl Measure for MyType {
///     fn measure(&self) -> Result<usize, EncodeError> {
///         EndianCodec::<u32>::le().measure(&self.value)
///     }
/// }
/// 
/// impl MeasureFixed for MyType {
///     fn measure_fixed() -> usize {
///         EndianCodec::<u32>::le().measure_fixed()
///     }
/// }
///
/// let codec = SelfCoded::new();
/// let my_type = MyType::new(0x12345678);
/// let mut data = vec![0u8; 4];
/// 
/// let mut offset = 0;
/// codec.encode(&my_type, &mut data, &mut offset).unwrap();
/// assert_eq!(data, vec![0x78, 0x56, 0x34, 0x12]);
/// assert_eq!(offset, 4);
/// 
/// let mut decode_offset = 0;
/// let decoded = codec.decode(&data, &mut decode_offset).unwrap();
/// assert_eq!(decoded.value, 0x12345678);
/// assert_eq!(decode_offset, 4);
/// 
/// let size = codec.measure(&my_type).unwrap();
/// assert_eq!(size, 4);
/// 
/// let fixed_size = codec.measure_fixed();
/// assert_eq!(fixed_size, 4);
/// ```
pub struct SelfCoded<T>(PhantomData<T>);

impl<T> SelfCoded<T> {
    /// Create a new `SelfCoded` instance.
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'encoded, 'decoded, T> Decoder<'encoded, 'decoded> for SelfCoded<T>
where
    T: Decode<'encoded> + 'decoded,
{
    type Decoded = T;
    fn decode(&self, encoded: &'encoded [u8], offset: &mut usize) -> Result<Self::Decoded, error::DecodeError> {
        T::decode(encoded, offset)
    }
}

impl<T> Encoder for SelfCoded<T>
where
    T: Encode,
{
    type Decoded = T;
    fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), error::EncodeError> {
        decoded.encode(encoded, offset)
    }
}

impl<T> Measurer for SelfCoded<T>
where
    T: Measure,
{
    type Decoded = T;
    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, error::EncodeError> {
        decoded.measure()
    }
}

impl<T> FixedMeasurer for SelfCoded<T>
where
    T: MeasureFixed,
{
    fn measure_fixed(&self) -> usize {
        T::measure_fixed()
    }
}
