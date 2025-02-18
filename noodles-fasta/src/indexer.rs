//! FASTA indexer.

use std::{
    error::Error,
    fmt,
    io::{self, BufRead},
};

use memchr::memchr;

use super::{
    fai::Record,
    reader::{read_line, DEFINITION_PREFIX, NEWLINE},
    record::definition::{Definition, ParseError},
};

/// A FASTA indexer.
pub struct Indexer<R> {
    inner: R,
    offset: u64,
    line_buf: Vec<u8>,
}

impl<R> Indexer<R>
where
    R: BufRead,
{
    /// Creates a FASTA indexer.
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            offset: 0,
            line_buf: Vec::new(),
        }
    }

    /// Consumes a single sequence line.
    ///
    /// If successful, this returns the number of bytes read from the stream (i.e., the line width)
    /// and the number of bases in the line. If the number of bytes read is 0, the entire sequence
    /// of the current record was read.
    fn consume_sequence_line(&mut self) -> io::Result<(usize, usize)> {
        self.line_buf.clear();

        let mut bytes_read = 0;
        let mut is_eol = false;

        loop {
            let buf = self.inner.fill_buf()?;

            if is_eol || buf.is_empty() || buf[0] == DEFINITION_PREFIX {
                break;
            }

            let len = match memchr(NEWLINE, buf) {
                Some(i) => {
                    self.line_buf.extend(&buf[..=i]);
                    is_eol = true;
                    i + 1
                }
                None => {
                    self.line_buf.extend(buf);
                    buf.len()
                }
            };

            self.inner.consume(len);

            bytes_read += len;
        }

        let base_count = len_with_right_trim(&self.line_buf);

        Ok((bytes_read, base_count))
    }

    /// Indexes a raw FASTA record.
    ///
    /// The position of the stream is expected to be at the start or at the start of another
    /// definition.
    ///
    /// # Errors
    ///
    /// An error is returned if the record fails to be completely read. This includes when
    ///
    ///   * the stream is not at the start of a definition;
    ///   * the record is missing a sequence;
    ///   * the sequence lines have a different number of bases, excluding the last line;
    ///   * or the sequence lines are not the same length, excluding the last line.
    pub fn index_record(&mut self) -> Result<Option<Record>, IndexError> {
        let definition = match self.read_definition() {
            Ok(None) => return Ok(None),
            Ok(Some(d)) => d,
            Err(e) => return Err(e.into()),
        };

        let offset = self.offset;
        let mut length = 0;

        let (line_width, line_bases) = self.consume_sequence_line()?;
        let (mut prev_line_width, mut prev_line_bases) = (line_width, line_bases);

        loop {
            self.offset += prev_line_width as u64;
            length += prev_line_bases;

            match self.consume_sequence_line() {
                Ok((0, _)) => break,
                Ok((bytes_read, base_count)) => {
                    if line_bases != prev_line_bases {
                        return Err(IndexError::InvalidLineBases(line_bases, prev_line_bases));
                    } else if line_width != prev_line_width {
                        return Err(IndexError::InvalidLineWidth(line_width, prev_line_width));
                    }

                    prev_line_width = bytes_read;
                    prev_line_bases = base_count;
                }
                Err(e) => return Err(IndexError::IoError(e)),
            }
        }

        if length == 0 {
            return Err(IndexError::EmptySequence(self.offset));
        }

        let record = Record::new(
            definition.name().into(),
            length as u64,
            offset,
            line_bases as u64,
            line_width as u64,
        );

        Ok(Some(record))
    }

    fn read_definition(&mut self) -> io::Result<Option<Definition>> {
        let mut buf = String::new();

        match read_line(&mut self.inner, &mut buf) {
            Ok(0) => return Ok(None),
            Ok(n) => self.offset += n as u64,
            Err(e) => return Err(e),
        }

        buf.parse()
            .map(Some)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

fn len_with_right_trim(vec: &[u8]) -> usize {
    match vec.iter().rposition(|x| !x.is_ascii_whitespace()) {
        Some(i) => i + 1,
        None => 0,
    }
}

#[derive(Debug)]
pub enum IndexError {
    EmptySequence(u64),
    InvalidDefinition(ParseError),
    InvalidLineBases(usize, usize),
    InvalidLineWidth(usize, usize),
    IoError(io::Error),
}

impl Error for IndexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EmptySequence(_) => None,
            Self::InvalidDefinition(e) => Some(e),
            Self::InvalidLineBases(..) => None,
            Self::InvalidLineWidth(..) => None,
            Self::IoError(e) => Some(e),
        }
    }
}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySequence(offset) => write!(f, "empty sequence at offset {}", offset),
            Self::InvalidDefinition(e) => e.fmt(f),
            Self::InvalidLineBases(expected, actual) => write!(
                f,
                "invalid line bases: expected {}, got {}",
                expected, actual
            ),
            Self::InvalidLineWidth(expected, actual) => write!(
                f,
                "invalid line width: expected {}, got {}",
                expected, actual
            ),
            Self::IoError(e) => e.fmt(f),
        }
    }
}

impl From<io::Error> for IndexError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<ParseError> for IndexError {
    fn from(error: ParseError) -> Self {
        Self::InvalidDefinition(error)
    }
}

impl From<IndexError> for io::Error {
    fn from(error: IndexError) -> Self {
        match error {
            IndexError::IoError(e) => e,
            _ => Self::new(io::ErrorKind::InvalidInput, error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_sequence_line() -> io::Result<()> {
        let data = b"ACGT\nNNNN\n";
        let mut indexer = Indexer::new(&data[..]);

        let (bytes_read, base_count) = indexer.consume_sequence_line()?;
        assert_eq!(bytes_read, 5);
        assert_eq!(base_count, 4);

        Ok(())
    }

    #[test]
    fn test_index_record() -> Result<(), IndexError> {
        let data = b">sq0\nACGT\n>sq1\nNNNN\nNNNN\nNN\n";
        let mut indexer = Indexer::new(&data[..]);

        let record = indexer.index_record()?;
        assert_eq!(record, Some(Record::new(String::from("sq0"), 4, 5, 4, 5)));

        let record = indexer.index_record()?;
        assert_eq!(record, Some(Record::new(String::from("sq1"), 10, 15, 4, 5)));

        assert!(indexer.index_record()?.is_none());

        Ok(())
    }

    #[test]
    fn test_index_record_with_invalid_line_bases() {
        let data = b">sq0\nACGT\nACG\nACGT\nAC\n";
        let mut indexer = Indexer::new(&data[..]);

        assert!(matches!(
            indexer.index_record(),
            Err(IndexError::InvalidLineBases(4, 3))
        ));
    }

    #[test]
    fn test_index_record_with_invalid_line_width() {
        let data = b">sq0\nACGT\nACGT \nACGT\nAC\n";
        let mut indexer = Indexer::new(&data[..]);

        assert!(matches!(
            indexer.index_record(),
            Err(IndexError::InvalidLineWidth(5, 6))
        ));
    }

    #[test]
    fn test_index_record_with_empty_seqeunce() {
        let data = b">sq0\n";
        let mut indexer = Indexer::new(&data[..]);

        assert!(matches!(
            indexer.index_record(),
            Err(IndexError::EmptySequence(5))
        ));
    }

    #[test]
    fn test_len_with_right_trim() {
        assert_eq!(len_with_right_trim(b"ATGC\n"), 4);
        assert_eq!(len_with_right_trim(b"ATGC\r\n"), 4);
    }
}
