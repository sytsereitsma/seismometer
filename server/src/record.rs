use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    InvalidNumber,
    InvalidUTF8,
    MissingField,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseErrorKind::InvalidNumber => write!(f, "Invalid number"),
            ParseErrorKind::InvalidUTF8 => write!(f, "Invalid UTF8"),
            ParseErrorKind::MissingField => write!(f, "Missing field"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseError {
    column: usize,
    kind: ParseErrorKind,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to parse column {} ({})", self.column, self.kind)
    }
}

#[derive(Clone)]
pub struct Record {
    pub timestamp: u64,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub x_filt: i32,
    pub y_filt: i32,
    pub z_filt: i32,
}

impl Record {
    pub fn default() -> Record {
        Record {
            timestamp: 0,
            x: 0,
            y: 0,
            z: 0,
            x_filt: 0,
            y_filt: 0,
            z_filt: 0,
        }
    }

    pub fn from<'a>(line: &[u8]) -> Result<Record, ParseError> {
        let mut parts = line.split(|&b| b == b',');

        let mut record = Record::default();
        record.timestamp = Record::parse_next(&mut parts, 0)?;
        record.x = Record::parse_next(&mut parts, 1)?;
        record.y = Record::parse_next(&mut parts, 2)?;
        record.z = Record::parse_next(&mut parts, 3)?;
        record.x_filt = Record::parse_next(&mut parts, 4)?;
        record.y_filt = Record::parse_next(&mut parts, 5)?;
        record.z_filt = Record::parse_next(&mut parts, 6)?;

        Ok(record)
    }

    fn parse_next<'a, 'b, I, T>(parts: &'b mut I, column: usize) -> Result<T, ParseError>
    where
        'a: 'b,
        I: Iterator<Item = &'a [u8]>,
        T: std::str::FromStr,
    {
        parts
            .next()
            .ok_or(ParseError {
                column: column,
                kind: ParseErrorKind::MissingField,
            })
            .and_then(|field| {
                std::str::from_utf8(field).map_err(|_| ParseError {
                    column: column,
                    kind: ParseErrorKind::InvalidUTF8,
                })
            })
            .and_then(|s| {
                s.parse::<T>().map_err(|_| ParseError {
                    column: column,
                    kind: ParseErrorKind::InvalidNumber,
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        let line = b"123456789,1,-2,3,-4,5,-6";
        let record = Record::from(line).unwrap();
        assert_eq!(record.timestamp, 123456789);
        assert_eq!(record.x, 1);
        assert_eq!(record.y, -2);
        assert_eq!(record.z, 3);
        assert_eq!(record.x_filt, -4);
        assert_eq!(record.y_filt, 5);
        assert_eq!(record.z_filt, -6);
    }

    #[test]
    fn test_from_missing_fields() {
        let line = b"123456789,1,-2,3,-4,5";
        let record = Record::from(line);
        assert!(record.is_err());
    }

    #[test]
    fn test_parse_next_invalid_number() {
        {
            // Negative unsigned number
            let line = b"-123456789";
            let mut parts = line.split(|&b| b == b',');
            let result: Result<u32, ParseError> = Record::parse_next(&mut parts, 3);
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert_eq!(err.column, 3);
            assert_eq!(err.kind, ParseErrorKind::InvalidNumber);
        }

        {
            // Invalid char
            let line = b"1a2";
            let mut parts = line.split(|&b| b == b',');
            let result: Result<i32, ParseError> = Record::parse_next(&mut parts, 4);
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert_eq!(err.column, 4);
            assert_eq!(err.kind, ParseErrorKind::InvalidNumber);
        }

        {
            // Missing field
            let line = b"";
            let mut parts = line.split(|&b| b == b',');
            let _ = parts.next().unwrap();
            let result: Result<i32, ParseError> = Record::parse_next(&mut parts, 0);
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert_eq!(err.column, 0);
            assert_eq!(err.kind, ParseErrorKind::MissingField);
        }

        {
            // Invalid UTF8
            let line = b"\xFF";
            let mut parts = line.split(|&b| b == b',');
            let result: Result<i32, ParseError> = Record::parse_next(&mut parts, 7);
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert_eq!(err.column, 7);
            assert_eq!(err.kind, ParseErrorKind::InvalidUTF8);
        }
    }
}
