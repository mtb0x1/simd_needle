/// Decodes a hex string into a vector of bytes
///
/// # Errors
/// Returns `FromHexError` if input contains invalid hex characters or has odd length
pub fn decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, FromHexError> {
    FromHex::try_from_hex(data)
}

/// Converts a hex character to its numeric value
fn val(c: u8, idx: usize) -> Result<u8, FromHexError> {
    match c {
        b'A'..=b'F' => Ok(c - b'A' + 10),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'0'..=b'9' => Ok(c - b'0'),
        _ => Err(FromHexError::InvalidHexCharacter {
            c: c as char,
            index: idx,
        }),
    }
}

/// Errors that can occur when decoding hex strings
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FromHexError {
    /// Invalid character found at specified index
    InvalidHexCharacter {
        c: char,
        index: usize,
    },
    /// Input string has an odd length
    OddLength,
    InvalidStringLength,
}

impl std::error::Error for FromHexError {}

impl std::fmt::Display for FromHexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            FromHexError::InvalidHexCharacter { c, index } => {
                write!(f, "Invalid character {:?} at position {}", c, index)
            }
            FromHexError::OddLength => write!(f, "Odd number of digits"),
            FromHexError::InvalidStringLength => write!(f, "Invalid string length"),
        }
    }
}

/// Trait for types that can be created from hex strings
trait FromHex: Sized {
    type Error;
    fn try_from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error>;
}

impl FromHex for Vec<u8> {
    type Error = FromHexError;

    fn try_from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let hex = hex.as_ref();
        if hex.len() % 2 != 0 {
            return Err(FromHexError::OddLength);
        }

        hex.chunks(2)
            .enumerate()
            .map(|(i, pair)| Ok(val(pair[0], 2 * i)? << 4 | val(pair[1], 2 * i + 1)?))
            .collect()
    }
}
