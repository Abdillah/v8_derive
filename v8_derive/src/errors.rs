use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Conversion failed; Expected Boolean")]
    ExpectedBoolean,
    #[error("Conversion failed; Expected Array")]
    ExpectedArray,
    #[error("Conversion failed; Expected Object")]
    ExpectedObject,
    #[error("Conversion failed {source}")]
    ConversionFailed {
        #[from]
        source: v8::DataError,
    },
    #[error("Field {0} not found")]
    FieldNoFound(String),
    #[error("Invalid field name: {0}")]
    InvalidField(String),
    #[error("Conversion failed; Expected String")]
    ExpectedString,
    #[error("Conversion failed; Expected Int32")]
    ExpectedI32,
    #[error("Conversion failed; Expected BigInt")]
    ExpectedI64,
    #[error("Conversion failed; Expected Number")]
    ExpectedF64,
    #[error("Conversion failed; Value out of range")]
    OutOfRange,
}

pub type Result<T> = std::result::Result<T, Error>;
