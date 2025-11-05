//! A binary codec framework for encoding and decoding Rust types to and from byte slices.
//!
//! It provides traits and implementations for codecs, encoders, decoders,
//! measurers, and fixed-size measurers.
//!
//! The library is designed to be extensible and efficient, allowing users to define
//! custom codecs for their types while leveraging existing implementations for common types.
//!
//! # Features
//! - `derive`: Enables procedural macros for deriving codec implementations for structs and enums.
//!
//! # Built-in Codecs
//! | Codec Type            | Default for | #[byten(...)] | Codes                                                                       |
//! |-----------------------|-------------|---------------------------------------------------------------------------------------------|
//! | [`U8Codec`]           | `u8`        | `u8`                        |  `u8` type                                                    |
//! | [`I8Codec`]           | `i8`        | `i8`                        |  `i8` type                                                    |
//! | [`U8ArrayCodec`]      | `[u8; N]`   | `[u8; N]`                   | fixed-size arrays of `u8` of size `N`                         |
//! | [`U8ArrayRefCodec`]   | `&[u8; N]`  | `&[u8; N]`                  |  references to fixed-size arrays of `u8` of size `N`          |
//! | [`BoolCodec`]         | `bool`      | `bool`                      | `bool` type                                                   |
//! | [`BoxCodec`]          | `Box<T>`    | `Box<T>` or `... box`       | `Box<T>` where `t_codec` is the codec for `T`                 |
//! | [`ArrayCodec`]        |             | `... $arr`                  | fixed-size arrays, `[Item; N]`                                |
//! | [`EndianCodec`]       |             | `... $be` or `... $le`      | primitive types with little/big endian byte orders            |
//! | [`SelfCoded`]         |             | `{SelfCoded::<T>::new()}`   | derived type `T`                                              |
//! | [`UTF8Codec`]         |             | `$utf8`                     | `&str` type using the provided bytes codec                    |
//! | [`CStrCodec`]         | `CStr`      | `CStr` or `&CStr`           | C-style strings (`CStr` and `&CStr`)                          |
//! | [`OwnedCodec`]        |             | `... $own`                  | owned data by cloning from borrowed data                      |
//! | [`VecCodec`]          |             | `... $vec[len]`             | `Vec<Item>` using the provided item and length codecs         |
//! | [`RemainingCodec`]    |             | `..`                        | all remaining bytes in the input                              |
//! | [`UVarBECodec`]       |             | `... $uvarbe`               | unsigned variable-length big-endian integers                  |
//! | [`OptionCodec`]       | `Option<T>` | `... $opt`                  | `Option<T>` using the provided codec for `T`                  |
//! | [`BytesCodec`]        |             | `$bytes[len]`               | byte slices with length prefixed by the provided codec        |
//! | [`PhantomCodec`]      |             | `= expr`                    | phantom codec with 0 size, codes the given constant value     |
//!

mod array;
mod asis;
mod endian;
mod error;
mod prelude;
mod selfcoded;
mod str;
mod util;
mod var;

use std::{ffi::CStr, ops::Deref};

#[cfg(feature = "derive")]
pub use byten_derive::{Decode, DecodeOwned, DefaultCodec, Encode, Measure, MeasureFixed, byten};

#[doc(inline)]
pub use array::*;
#[doc(inline)]
pub use asis::*;
#[doc(inline)]
pub use endian::*;
#[doc(inline)]
pub use error::*;
#[doc(inline)]
pub use prelude::*;
#[doc(inline)]
pub use selfcoded::*;
#[doc(inline)]
pub use str::*;
#[doc(inline)]
pub use util::*;
#[doc(inline)]
pub use var::*;

// codec traits

/// A codec that can encode values of type `Decoded` into a byte slice.
/// It returns a result indicating success or failure but never panics.
///
/// It takes a reference to the value to be encoded, a mutable byte slice to write the encoded data into and an offset
/// indicating the current position in the byte slice.
/// The offset is updated to point to the next position after the encoded data.
///
/// # Examples
/// ## Simple u32-big-endian encoder
/// ```
/// use byten::{Encoder, EncodeError};
///
/// // Define a simple encoder for u32 in big-endian format
/// struct U32BigEndianCodec;
/// impl Encoder for U32BigEndianCodec {
///     type Decoded = u32;
///     fn encode(&self, decoded: &Self::Decoded, encoded: &mut [u8], offset: &mut usize) -> Result<(), EncodeError> {
///         // Check if there is enough space in the buffer to avoid panicking
///         if *offset + 4 > encoded.len() {
///             return Err(EncodeError::BufferTooSmall);
///         }
///         // Encode the u32 in big-endian format and write it to the buffer at the current offset
///         let [b0, b1, b2, b3] = decoded.to_be_bytes();
///         encoded[*offset] = b0;
///         encoded[*offset + 1] = b1;
///         encoded[*offset + 2] = b2;
///         encoded[*offset + 3] = b3;
///         // Update the offset to point to the next position
///         *offset += 4;
///         Ok(())
///     }
/// }
///
/// let codec = U32BigEndianCodec;
/// let mut buffer = [0xFFu8; 6];
/// let mut offset = 1; // start encoding into at index 1
/// codec.encode(&0x12345678, &mut buffer, &mut offset).unwrap();
/// assert_eq!(buffer, [
///     0xFF, // stays unchanged
///     0x12, 0x34, 0x56, 0x78, // encoded u32
///     0xFF, // stays unchanged
/// ]);
/// assert_eq!(offset, 5); // offset updated correctly
/// ```
pub trait Encoder {
    type Decoded: ?Sized;
    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), error::EncodeError>;
}

impl<T, Ref> Encoder for Ref
where
    T: Encoder,
    Ref: Deref<Target = T>,
{
    type Decoded = T::Decoded;
    fn encode(
        &self,
        decoded: &Self::Decoded,
        encoded: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), error::EncodeError> {
        self.deref().encode(decoded, encoded, offset)
    }
}

/// A codec that can decode values of type `Decoded` from a byte slice.
/// It returns a result containing the decoded value or an error if decoding fails.
///
/// It takes a byte slice containing the encoded data and a mutable offset indicating the current position in
/// the byte slice.
/// The offset is updated to point to the next position after the decoded data.
///
/// The decoded value has a lifetime tied to the lifetime of the encoded byte slice.
/// It is important to ensure that the byte slice remains valid for the duration of the decoded value's usage.
/// This is not relevant for types that own their data (e.g., `Vec<u8>`, `String`), but is crucial for
/// types that borrow data from the byte slice (e.g., `&[u8]`, `&str`).
///
/// # Examples
/// ## Owned decoded value
/// ```
/// use byten::{Decoder, DecodeError};
///
/// // Define a simple decoder for u32 in big-endian format
/// struct U32BigEndianCodec;
/// impl Decoder<'_, '_> for U32BigEndianCodec {
///     type Decoded = u32;
///     fn decode(&self, encoded: &[u8], offset: &mut usize) -> Result<Self::Decoded, DecodeError> {
///         // Check if there is enough data in the buffer to avoid panicking
///         if *offset + 4 > encoded.len() {
///             return Err(DecodeError::EOF);
///         }
///         // Decode the u32 from big-endian format from the buffer at the current offset
///         let value = u32::from_be_bytes([
///             encoded[*offset],
///             encoded[*offset + 1],
///             encoded[*offset + 2],
///             encoded[*offset + 3],
///         ]);
///         // Update the offset to point to the next position
///         *offset += 4;
///         Ok(value)
///     }
/// }
///
/// let codec = U32BigEndianCodec;
/// let buffer = [0xFFu8, 0x12, 0x34, 0x56, 0x78, 0xFF];
/// let mut offset = 1; // start decoding from index 1
/// let value = codec.decode(&buffer, &mut offset).unwrap();
/// assert_eq!(value, 0x12345678);
/// assert_eq!(offset, 5); // offset updated correctly
/// ```
///
/// ## Borrowed decoded value
/// ```
/// use byten::{Decoder, DecodeError};
///
/// // Define a simple decoder for &str assuming UTF-8 encoding
/// struct StrCodec;
/// impl<'encoded: 'decoded, 'decoded> Decoder<'encoded, 'decoded> for StrCodec {
///     type Decoded = &'decoded str;
///     fn decode(&self, encoded: &'encoded [u8], offset: &mut usize) -> Result<Self::Decoded, DecodeError> {
///         // For simplicity, assume the string is null-terminated
///         let end = encoded[*offset..]
///             .iter()
///             .position(|&b| b == 0)
///             .ok_or(DecodeError::InvalidData)?
///             + *offset;
///         let s = std::str::from_utf8(&encoded[*offset..end]).map_err(|_| DecodeError::InvalidData)?;
///         *offset = end + 1; // move past the null terminator
///         Ok(s)
///     }
/// }
///
/// let codec = StrCodec;
/// let buffer = [b'H', b'e', b'l', b'l', b'o', 0, b'W', b'o', b'r', b'l', b'd', b'!', 0];
/// let mut offset = 0;
/// let first = codec.decode(&buffer, &mut offset).unwrap();
/// assert_eq!(first, "Hello");
/// assert_eq!(offset, 6);
/// let second = codec.decode(&buffer, &mut offset).unwrap();
/// assert_eq!(second, "World!");
/// assert_eq!(offset, 13);
/// ```
pub trait Decoder<'encoded, 'decoded> {
    type Decoded: 'decoded;
    fn decode(
        &self,
        encoded: &'encoded [u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, error::DecodeError>;
}

impl<'encoded, 'decoded, T, Ref> Decoder<'encoded, 'decoded> for Ref
where
    T: Decoder<'encoded, 'decoded>,
    Ref: Deref<Target = T>,
{
    type Decoded = T::Decoded;
    fn decode(
        &self,
        encoded: &'encoded [u8],
        offset: &mut usize,
    ) -> Result<Self::Decoded, error::DecodeError> {
        self.deref().decode(encoded, offset)
    }
}

/// A codec that can measure the size in bytes required to encode a value of type `Decoded`.
/// It returns a result containing the size in bytes or an error if measurement fails.
///
/// It takes a reference to the value to be measured.
///
/// This is particularly useful for types with variable-length encoding
/// where the size cannot be determined statically.
/// This is also useful for pre-allocating buffers of the correct size before encoding.
///
/// # Examples
/// ## Simple variable-length string measurer
/// ```
/// use byten::{Measurer, EncodeError};
///
/// // Define a simple measurer for strings that encodes the length as a u8 prefix
/// struct VarLenStrMeasurer;
/// impl Measurer for VarLenStrMeasurer {
///     type Decoded = str;
///     fn measure(&self, decoded: &Self::Decoded) -> Result<usize, EncodeError> {
///         let len = decoded.len();
///         if len > 255 {
///             return Err(EncodeError::InvalidData);
///         }
///         Ok(1 + len) // 1 byte for length prefix + string bytes
///     }
/// }
///
/// let measurer = VarLenStrMeasurer;
/// let size = measurer.measure("Hello").unwrap();
/// assert_eq!(size, 6); // 1 byte for length + 5 bytes for "Hello"
/// ```
pub trait Measurer {
    type Decoded: ?Sized;
    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, error::EncodeError>;
}

impl<T, Ref> Measurer for Ref
where
    T: Measurer,
    Ref: Deref<Target = T>,
{
    type Decoded = T::Decoded;
    fn measure(&self, decoded: &Self::Decoded) -> Result<usize, error::EncodeError> {
        self.deref().measure(decoded)
    }
}

/// A codec that can measure the size in bytes required to encode a value of type `Decoded`
/// where the size is fixed and does not depend on the actual value.
/// It provides a method to get the fixed size directly without needing a value.
///
/// It is expected to return the same size for any value of type `Decoded`
/// unless the internal configuration of the codec changes.
///
/// This is particularly useful for types with fixed-length encoding
/// where the size is known at compile time.
///
/// This can improve performance by avoiding the need to pass a value
/// when the size is constant by allowing the compiler to optimize for it.
///
/// # Examples
/// ## Simple fixed-length u32 measurer
/// ```
/// use byten::{FixedMeasurer, Measurer, EncodeError};
///
/// // Define a simple fixed-length measurer for u32
/// struct U32FixedMeasurer;
/// impl FixedMeasurer for U32FixedMeasurer {
///     fn measure_fixed(&self) -> usize {
///         4 // u32 is always 4 bytes regardless of its value or endianness
///     }
/// }
///
/// // FixedMeasurer also requires Measurer implementation
/// impl Measurer for U32FixedMeasurer {
///    type Decoded = u32;
///    fn measure(&self, _decoded: &Self::Decoded) -> Result<usize, EncodeError> {
///       // For fixed-length types, the measured size is always the fixed size
///       Ok(self.measure_fixed())
///   }
/// }
///
/// let measurer = U32FixedMeasurer;
/// let size = measurer.measure_fixed();
/// assert_eq!(size, 4);
/// ```
pub trait FixedMeasurer: Measurer {
    fn measure_fixed(&self) -> usize;
}

impl<T, Ref> FixedMeasurer for Ref
where
    T: FixedMeasurer,
    Ref: Deref<Target = T>,
{
    fn measure_fixed(&self) -> usize {
        self.deref().measure_fixed()
    }
}

// default codec

/// A trait for types that have a default codec associated with them.
/// This allows for easy retrieval of the default codec for a type
/// without needing to specify the codec explicitly each time.
///
/// # Examples
/// ```
/// use byten::{DefaultCodec, U8Codec};
///
/// struct MyType {
///     // ..
/// };
///
/// struct MyCodec;
///
/// impl DefaultCodec for MyType {
///     type Codec = MyCodec;
///     fn default_codec() -> Self::Codec { MyCodec }
/// }
///
/// let codec = MyType::default_codec();
/// // codec is of type MyCodec
/// ```
pub trait DefaultCodec {
    type Codec;
    fn default_codec() -> Self::Codec;
}

// defaults

impl DefaultCodec for u8 {
    type Codec = U8Codec;
    fn default_codec() -> Self::Codec {
        U8Codec
    }
}

impl DefaultCodec for i8 {
    type Codec = I8Codec;
    fn default_codec() -> Self::Codec {
        I8Codec
    }
}

impl<const N: usize> DefaultCodec for [u8; N] {
    type Codec = U8ArrayCodec<N>;
    fn default_codec() -> Self::Codec {
        U8ArrayCodec
    }
}

impl<const N: usize> DefaultCodec for &[u8; N] {
    type Codec = U8ArrayRefCodec<N>;
    fn default_codec() -> Self::Codec {
        U8ArrayRefCodec::<N>
    }
}

impl DefaultCodec for bool {
    type Codec = BoolCodec;
    fn default_codec() -> Self::Codec {
        BoolCodec
    }
}

impl<T> DefaultCodec for Box<T>
where
    T: DefaultCodec + ?Sized,
{
    type Codec = BoxCodec<T::Codec>;
    fn default_codec() -> Self::Codec {
        BoxCodec(T::default_codec())
    }
}

impl DefaultCodec for CStr {
    type Codec = CStrCodec;
    fn default_codec() -> Self::Codec {
        CStrCodec
    }
}

impl DefaultCodec for &CStr {
    type Codec = CStrCodec;
    fn default_codec() -> Self::Codec {
        CStrCodec
    }
}

impl<T> DefaultCodec for Option<T>
where
    T: DefaultCodec,
{
    type Codec = OptionCodec<T::Codec>;
    fn default_codec() -> Self::Codec {
        OptionCodec::new(T::default_codec())
    }
}
