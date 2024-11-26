/// Telemetry message generation library.
///
/// *Models* generate streams of telemetry messages by implementing [`TelemStream`].
/// *Protocols* are used to generate specific message formats and encodings, by implementing
/// [`TelemMsg`].
use std::sync::Once;

use thiserror::Error;

pub mod coord;
pub mod model;
pub mod protocol;

/// Result type for this library
pub type TGResult<T> = std::result::Result<T, Error>;

/// Error type for this library
#[derive(Debug, Error)]
pub enum Error {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Invalid coordinate: {0}")]
    InvalidCoord(String),
}

impl From<nom::error::Error<&[u8]>> for Error {
    fn from(err: nom::error::Error<&[u8]>) -> Self {
        Error::ParseError(format!("{:?}", err))
    }
}

/// Test binary helper to init tracing. This is usually the responsibility of the consumer of the
/// library crate.
pub fn lazy_init_tracing() {
    {
        static INIT: Once = Once::new();
        &INIT
    }
    .call_once(|| {
        tracing_subscriber::fmt::init();
    });
}
