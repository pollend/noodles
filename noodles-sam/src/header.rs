//! SAM header and records.
//!
//! A SAM header is a list of header records. There are 5 record types: [header] (`@HD`),
//! [reference sequence] (`@SQ`), [read group] (`@RG`), [program] (`@PG`), and comment (`@CO`).
//!
//! Each record is effectively a map. It defines key-value pairs associated with that record type.
//!
//! All records are optional, which means an empty header is considered a valid SAM header.
//!
//! If there is a header record, it must appear as the first record.
//!
//! Reference sequence, read group, program, and comment records are lists of records of the same
//! type. Reference sequences must be ordered; whereas read groups, programs, and comments can be
//! unordered. (`sam::Header` defines them to be ordered.)
//!
//! [header]: `header::Header`
//! [reference sequence]: `ReferenceSequence`
//! [read group]: `ReadGroup`
//! [program]: `Program`
//!
//! # Examples
//!
//! ## Parse a SAM header
//!
//! ```
//! use noodles_sam as sam;
//!
//! let s = "\
//! @HD\tVN:1.6\tSO:coordinate
//! @SQ\tSN:sq0\tLN:8
//! @SQ\tSN:sq1\tLN:13
//! ";
//!
//! let header: sam::Header = s.parse()?;
//!
//! assert!(header.header().is_some());
//! assert_eq!(header.reference_sequences().len(), 2);
//! assert!(header.read_groups().is_empty());
//! assert!(header.programs().is_empty());
//! assert!(header.comments().is_empty());
//! # Ok::<(), sam::header::ParseError>(())
//! ```
//!
//! ## Create a SAM header
//!
//! ```
//! use noodles_sam::{self as sam, header::{self, ReferenceSequence}};
//!
//! let header = sam::Header::builder()
//!     .set_header(header::header::Header::default())
//!     .add_reference_sequence(ReferenceSequence::new("sq0".parse()?, 8)?)
//!     .add_reference_sequence(ReferenceSequence::new("sq1".parse()?, 13)?)
//!     .build();
//!
//! assert!(header.header().is_some());
//! assert_eq!(header.reference_sequences().len(), 2);
//! assert!(header.read_groups().is_empty());
//! assert!(header.programs().is_empty());
//! assert!(header.comments().is_empty());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod builder;
#[allow(clippy::module_inception)]
pub mod header;
mod parser;
pub mod program;
pub mod read_group;
pub mod record;
pub mod reference_sequence;

use std::{fmt, str::FromStr};

use indexmap::IndexMap;

pub use self::{
    builder::Builder, parser::ParseError, program::Program, read_group::ReadGroup,
    reference_sequence::ReferenceSequence,
};

pub use self::record::Record;

/// A reference seqeuence dictionary.
pub type ReferenceSequences = IndexMap<String, ReferenceSequence>;

/// An ordered map of read groups.
pub type ReadGroups = IndexMap<String, ReadGroup>;

/// An ordered map of programs.
pub type Programs = IndexMap<String, Program>;

/// A SAM header.
///
/// Records are grouped by their types: header, reference seqeuence, read group, program, and
/// comment.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Header {
    header: Option<header::Header>,
    reference_sequences: ReferenceSequences,
    read_groups: ReadGroups,
    programs: Programs,
    comments: Vec<String>,
}

impl Header {
    /// Returns a builder to create a SAM header.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    /// let builder = sam::Header::builder();
    /// ```
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Returns the SAM header header if it is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::{self as sam, header};
    ///
    /// let header = sam::Header::default();
    /// assert!(header.header().is_none());
    ///
    /// let header = sam::Header::builder()
    ///     .set_header(header::header::Header::default())
    ///     .build();
    ///
    /// assert!(header.header().is_some());
    /// ```
    pub fn header(&self) -> Option<&header::Header> {
        self.header.as_ref()
    }

    /// Returns a mutable reference to the SAM header header if it is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::{self as sam, header::{self, header::Version}};
    ///
    /// let mut header = sam::Header::builder()
    ///     .set_header(header::header::Header::new(Version::new(1, 6)))
    ///     .build();
    /// assert_eq!(header.header().map(|h| h.version()), Some(Version::new(1, 6)));
    ///
    /// header.header_mut().as_mut().map(|h| {
    ///     *h.version_mut() = Version::new(1, 5)
    /// });
    /// assert_eq!(header.header().map(|h| h.version()), Some(Version::new(1, 5)));
    ///
    /// *header.header_mut() = None;
    /// assert!(header.header().is_none());
    /// ```
    pub fn header_mut(&mut self) -> &mut Option<header::Header> {
        &mut self.header
    }

    /// Returns the SAM header reference sequences.
    ///
    /// This is also called the reference sequence dictionary.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::{self as sam, header::ReferenceSequence};
    ///
    /// let header = sam::Header::builder()
    ///     .add_reference_sequence(ReferenceSequence::new("sq0".parse()?, 13)?)
    ///     .build();
    ///
    /// let reference_sequences = header.reference_sequences();
    /// assert_eq!(reference_sequences.len(), 1);
    /// assert!(reference_sequences.contains_key("sq0"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn reference_sequences(&self) -> &ReferenceSequences {
        &self.reference_sequences
    }

    /// Returns a mutable reference to the SAM header reference sequences.
    ///
    /// This is also called the reference sequence dictionary.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::{self as sam, header::ReferenceSequence};
    ///
    /// let mut header = sam::Header::default();
    /// assert!(header.reference_sequences().is_empty());
    ///
    /// let reference_sequence = ReferenceSequence::new("sq0".parse()?, 13)?;
    ///
    /// header.reference_sequences_mut().insert(
    ///     reference_sequence.name().to_string(),
    ///     reference_sequence,
    /// );
    ///
    /// let reference_sequences = header.reference_sequences();
    /// assert_eq!(reference_sequences.len(), 1);
    /// assert!(reference_sequences.contains_key("sq0"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn reference_sequences_mut(&mut self) -> &mut ReferenceSequences {
        &mut self.reference_sequences
    }

    /// Returns the SAM header read groups.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::{self as sam, header::ReadGroup};
    ///
    /// let header = sam::Header::builder()
    ///     .add_read_group(ReadGroup::new("rg0"))
    ///     .build();
    ///
    /// let read_groups = header.read_groups();
    /// assert_eq!(read_groups.len(), 1);
    /// assert!(read_groups.contains_key("rg0"));
    /// ```
    pub fn read_groups(&self) -> &ReadGroups {
        &self.read_groups
    }

    /// Returns a mutable reference to the SAM header read groups.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::{self as sam, header::ReadGroup};
    ///
    /// let mut header = sam::Header::default();
    /// assert!(header.read_groups().is_empty());
    ///
    /// header.read_groups_mut().insert(
    ///     String::from("rg0"),
    ///     ReadGroup::new("rg0"),
    /// );
    ///
    /// let read_groups = header.read_groups();
    /// assert_eq!(read_groups.len(), 1);
    /// assert!(read_groups.contains_key("rg0"));
    /// ```
    pub fn read_groups_mut(&mut self) -> &mut ReadGroups {
        &mut self.read_groups
    }

    /// Returns the SAM header programs.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::{self as sam, header::Program};
    ///
    /// let header = sam::Header::builder()
    ///     .add_program(Program::new("noodles-sam"))
    ///     .build();
    ///
    /// let programs = header.programs();
    /// assert_eq!(programs.len(), 1);
    /// assert!(programs.contains_key("noodles-sam"));
    /// ```
    pub fn programs(&self) -> &Programs {
        &self.programs
    }

    /// Returns a mutable reference to the SAM header programs.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::{self as sam, header::Program};
    ///
    /// let mut header = sam::Header::default();
    ///
    /// header.programs_mut().insert(
    ///     String::from("noodles-sam"),
    ///     Program::new("noodles-sam"),
    /// );
    ///
    /// let programs = header.programs();
    /// assert_eq!(programs.len(), 1);
    /// assert!(programs.contains_key("noodles-sam"));
    /// ```
    pub fn programs_mut(&mut self) -> &mut Programs {
        &mut self.programs
    }

    /// Returns the SAM header comments.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    /// let header = sam::Header::builder().add_comment("noodles-sam").build();
    /// let comments = header.comments();
    /// assert_eq!(comments.len(), 1);
    /// assert_eq!(&comments[0], "noodles-sam");
    /// ```
    pub fn comments(&self) -> &[String] {
        &self.comments
    }

    /// Returns a mutable reference to the SAM header comments.
    ///
    /// To simply append a comment record, consider using [`Self::add_comment`] instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    ///
    /// let mut header = sam::Header::default();
    /// header.comments_mut().push(String::from("noodles-sam"));
    ///
    /// let comments = header.comments();
    /// assert_eq!(comments.len(), 1);
    /// assert_eq!(&comments[0], "noodles-sam");
    /// ```
    pub fn comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.comments
    }

    /// Adds a comment.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    ///
    /// let mut header = sam::Header::default();
    /// header.add_comment("noodles-sam");
    ///
    /// let comments = header.comments();
    /// assert_eq!(comments.len(), 1);
    /// assert_eq!(&comments[0], "noodles-sam");
    /// ```
    pub fn add_comment<S>(&mut self, comment: S)
    where
        S: Into<String>,
    {
        self.comments.push(comment.into());
    }

    /// Returns whether there are no records in this SAM header.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    ///
    /// let header = sam::Header::default();
    /// assert!(header.is_empty());
    ///
    /// let header = sam::Header::builder().add_comment("noodles-sam").build();
    /// assert!(!header.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.header.is_none()
            && self.reference_sequences.is_empty()
            && self.read_groups.is_empty()
            && self.programs.is_empty()
            && self.comments.is_empty()
    }

    /// Removes all records from the header.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    ///
    /// let mut header = sam::Header::builder().add_comment("ndls").build();
    /// assert!(!header.is_empty());
    ///
    /// header.clear();
    /// assert!(header.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.header.take();
        self.reference_sequences.clear();
        self.read_groups.clear();
        self.programs.clear();
        self.comments.clear();
    }
}

impl fmt::Display for Header {
    /// Formats the SAM header as a raw SAM header.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::{self as sam, header::{self, ReferenceSequence}};
    ///
    /// let header = sam::Header::builder()
    ///     .set_header(header::header::Header::new(header::header::Version::new(1, 6)))
    ///     .add_reference_sequence(ReferenceSequence::new("sq0".parse()?, 8)?)
    ///     .add_reference_sequence(ReferenceSequence::new("sq1".parse()?, 13)?)
    ///     .build();
    ///
    /// let expected = "\
    /// @HD\tVN:1.6
    /// @SQ\tSN:sq0\tLN:8
    /// @SQ\tSN:sq1\tLN:13
    /// ";
    ///
    /// assert_eq!(header.to_string(), expected);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(header) = self.header() {
            writeln!(f, "{}", header)?;
        }

        for reference_sequence in self.reference_sequences.values() {
            writeln!(f, "{}", reference_sequence)?;
        }

        for read_group in self.read_groups.values() {
            writeln!(f, "{}", read_group)?;
        }

        for program in self.programs.values() {
            writeln!(f, "{}", program)?;
        }

        for comment in &self.comments {
            writeln!(f, "{}\t{}", record::Kind::Comment, comment)?;
        }

        Ok(())
    }
}

impl FromStr for Header {
    type Err = ParseError;

    /// Parses a raw SAM header.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    ///
    /// let s = "\
    /// @HD\tVN:1.6\tSO:coordinate
    /// @SQ\tSN:sq0\tLN:8
    /// @SQ\tSN:sq1\tLN:13
    /// ";
    ///
    /// let header: sam::Header = s.parse()?;
    ///
    /// assert!(header.header().is_some());
    /// assert_eq!(header.reference_sequences().len(), 2);
    /// assert!(header.read_groups().is_empty());
    /// assert!(header.programs().is_empty());
    /// assert!(header.comments().is_empty());
    /// # Ok::<(), sam::header::ParseError>(())
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt() -> Result<(), Box<dyn std::error::Error>> {
        let header = Header::builder()
            .set_header(header::Header::new(header::Version::new(1, 6)))
            .add_reference_sequence(ReferenceSequence::new("sq0".parse()?, 8)?)
            .add_reference_sequence(ReferenceSequence::new("sq1".parse()?, 13)?)
            .add_read_group(ReadGroup::new("rg0"))
            .add_read_group(ReadGroup::new("rg1"))
            .add_program(Program::new("pg0"))
            .add_program(Program::new("pg1"))
            .add_comment("noodles")
            .add_comment("sam")
            .build();

        let actual = header.to_string();
        let expected = "\
@HD\tVN:1.6
@SQ\tSN:sq0\tLN:8
@SQ\tSN:sq1\tLN:13
@RG\tID:rg0
@RG\tID:rg1
@PG\tID:pg0
@PG\tID:pg1
@CO\tnoodles
@CO\tsam
";

        assert_eq!(actual, expected);

        Ok(())
    }
}
