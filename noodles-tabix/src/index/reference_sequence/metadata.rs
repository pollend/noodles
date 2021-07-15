//! Tabix reference sequence metadata.

use noodles_bgzf::VirtualPosition;

/// Tabix reference sequence metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Metadata {
    start_position: VirtualPosition,
    end_position: VirtualPosition,
    mapped_record_count: u64,
    unmapped_record_count: u64,
}

impl Metadata {
    /// Creates reference sequence metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bgzf as bgzf;
    /// use noodles_tabix::index::reference_sequence::Metadata;
    ///
    /// let metadata = Metadata::new(
    ///     bgzf::VirtualPosition::from(610),
    ///     bgzf::VirtualPosition::from(1597),
    ///     55,
    ///     0,
    /// );
    /// ```
    pub fn new(
        start_position: VirtualPosition,
        end_position: VirtualPosition,
        mapped_record_count: u64,
        unmapped_record_count: u64,
    ) -> Self {
        Self {
            start_position,
            end_position,
            mapped_record_count,
            unmapped_record_count,
        }
    }

    /// Returns the start virtual position.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bgzf as bgzf;
    /// use noodles_tabix::index::reference_sequence::Metadata;
    ///
    /// let metadata = Metadata::new(
    ///     bgzf::VirtualPosition::from(610),
    ///     bgzf::VirtualPosition::from(1597),
    ///     55,
    ///     0,
    /// );
    ///
    /// assert_eq!(metadata.start_position(), bgzf::VirtualPosition::from(610));
    /// ```
    pub fn start_position(&self) -> VirtualPosition {
        self.start_position
    }

    /// Returns the end virtual position.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bgzf as bgzf;
    /// use noodles_tabix::index::reference_sequence::Metadata;
    ///
    /// let metadata = Metadata::new(
    ///     bgzf::VirtualPosition::from(610),
    ///     bgzf::VirtualPosition::from(1597),
    ///     55,
    ///     0,
    /// );
    ///
    /// assert_eq!(metadata.end_position(), bgzf::VirtualPosition::from(1597));
    /// ```
    pub fn end_position(&self) -> VirtualPosition {
        self.end_position
    }

    /// Returns the number of mapped records.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bgzf as bgzf;;
    /// use noodles_tabix::index::reference_sequence::Metadata;
    ///
    /// let metadata = Metadata::new(
    ///     bgzf::VirtualPosition::from(610),
    ///     bgzf::VirtualPosition::from(1597),
    ///     55,
    ///     0,
    /// );
    ///
    /// assert_eq!(metadata.mapped_record_count(), 55);
    /// ```
    pub fn mapped_record_count(&self) -> u64 {
        self.mapped_record_count
    }

    /// Returns the number of unmapped records.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_bgzf as bgzf;;
    /// use noodles_tabix::index::reference_sequence::Metadata;
    ///
    /// let metadata = Metadata::new(
    ///     bgzf::VirtualPosition::from(610),
    ///     bgzf::VirtualPosition::from(1597),
    ///     55,
    ///     0,
    /// );
    ///
    /// assert_eq!(metadata.unmapped_record_count(), 0);
    /// ```
    pub fn unmapped_record_count(&self) -> u64 {
        self.unmapped_record_count
    }
}
