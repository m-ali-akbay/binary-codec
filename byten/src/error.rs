use core::convert::Infallible;
use core::num::TryFromIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("End of file reached")]
    EOF,

    #[error("Invalid discriminant")]
    InvalidDiscriminant,

    #[error("Invalid usize")]
    InvalidUSize,

    #[error("Data conversion failure")]
    ConversionFailure,

    #[error("Invalid data")]
    InvalidData,

    #[error("Codec failure")]
    CodecFailure,

    #[error("Bit overflow")]
    BitOverflow,

    #[cfg(feature = "anyhow")]
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl From<Infallible> for DecodeError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<TryFromIntError> for DecodeError {
    fn from(_: TryFromIntError) -> Self {
        DecodeError::CodecFailure
    }
}

#[derive(Error, Debug)]
pub enum EncodeError {
    #[error("Buffer too small")]
    BufferTooSmall,

    #[error("Invalid usize")]
    InvalidUSize,

    #[error("Data conversion failure")]
    CodecFailure,

    #[error("Invalid data")]
    InvalidData,

    #[error("Bit overflow")]
    BitOverflow,

    #[cfg(feature = "anyhow")]
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl From<Infallible> for EncodeError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<TryFromIntError> for EncodeError {
    fn from(_: TryFromIntError) -> Self {
        EncodeError::CodecFailure
    }
}
