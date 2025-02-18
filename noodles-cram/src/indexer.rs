use std::{cmp, collections::HashMap, fs::File, io, path::Path};

use noodles_core::Position;
use noodles_sam::AlignmentRecord;

use super::{
    crai,
    data_container::{slice, CompressionHeader, Slice},
    Reader,
};

/// Indexes a CRAM file.
///
/// # Examples
///
/// ```no_run
/// # use std::io;
/// use noodles_cram as cram;
/// let index = cram::index("sample.cram")?;
/// # Ok::<(), io::Error>(())
/// ```
pub fn index<P>(src: P) -> io::Result<crai::Index>
where
    P: AsRef<Path>,
{
    let mut reader = File::open(src).map(Reader::new)?;
    reader.read_file_definition()?;
    reader.read_file_header()?;

    let mut index = Vec::new();
    let mut container_position = reader.position()?;

    while let Some((container_header, data_container)) =
        reader.read_data_container_with_container_header()?
    {
        let container_len = container_header.len();

        let landmarks = container_header.landmarks();
        let slice_count = landmarks.len();

        for (i, slice) in data_container.slices().iter().enumerate() {
            let landmark = landmarks[i];

            let slice_length = if i < slice_count - 1 {
                landmarks[i + 1] - landmark
            } else {
                container_len - landmark
            };

            push_index_records(
                &mut index,
                data_container.compression_header(),
                slice,
                container_position,
                landmark as u64,
                slice_length as u64,
            )?;
        }

        container_position = reader.position()?;
    }

    Ok(index)
}

fn push_index_records(
    index: &mut crai::Index,
    compression_header: &CompressionHeader,
    slice: &Slice,
    container_position: u64,
    landmark: u64,
    slice_length: u64,
) -> io::Result<()> {
    if slice.header().reference_sequence_id().is_many() {
        push_index_records_for_multi_reference_slice(
            index,
            compression_header,
            slice,
            container_position,
            landmark,
            slice_length,
        )
    } else {
        push_index_record_for_single_reference_slice(
            index,
            slice.header(),
            container_position,
            landmark,
            slice_length,
        )
    }
}

#[derive(Debug)]
struct SliceReferenceSequenceAlignmentRangeInclusive {
    start: Option<Position>,
    end: Option<Position>,
}

impl Default for SliceReferenceSequenceAlignmentRangeInclusive {
    fn default() -> Self {
        Self {
            start: Position::new(usize::MAX),
            end: None,
        }
    }
}

fn push_index_records_for_multi_reference_slice(
    index: &mut crai::Index,
    compression_header: &CompressionHeader,
    slice: &Slice,
    container_position: u64,
    landmark: u64,
    slice_length: u64,
) -> io::Result<()> {
    let mut reference_sequence_ids: HashMap<
        Option<usize>,
        SliceReferenceSequenceAlignmentRangeInclusive,
    > = HashMap::new();

    for record in slice.records(compression_header)? {
        let reference_sequence_id = record.reference_sequence_id();

        let range = reference_sequence_ids
            .entry(reference_sequence_id)
            .or_default();

        let alignment_start = record.alignment_start();
        range.start = cmp::min(range.start, alignment_start);

        let alignment_end = record.alignment_end();
        range.end = cmp::max(range.end, alignment_end);
    }

    let mut sorted_reference_sequence_ids: Vec<_> =
        reference_sequence_ids.keys().copied().collect();
    sorted_reference_sequence_ids.sort_unstable();

    for reference_sequence_id in sorted_reference_sequence_ids {
        let (alignment_start, alignment_span) = if reference_sequence_id.is_some() {
            let range = &reference_sequence_ids[&reference_sequence_id];

            if let (Some(start), Some(end)) = (range.start, range.end) {
                let span = usize::from(end) - usize::from(start) + 1;
                (Some(start), span)
            } else {
                todo!("unhandled interval: {:?}", range);
            }
        } else {
            (None, 0)
        };

        let record = crai::Record::new(
            reference_sequence_id,
            alignment_start,
            alignment_span,
            container_position,
            landmark as u64,
            slice_length as u64,
        );

        index.push(record);
    }

    Ok(())
}

fn push_index_record_for_single_reference_slice(
    index: &mut crai::Index,
    slice_header: &slice::Header,
    container_position: u64,
    landmark: u64,
    slice_length: u64,
) -> io::Result<()> {
    use crate::container::ReferenceSequenceId;

    let slice_reference_sequence_id = slice_header.reference_sequence_id();

    let (reference_sequence_id, alignment_start, alignment_span) = match slice_reference_sequence_id
    {
        ReferenceSequenceId::Some(id) => {
            let reference_sequence_id = usize::try_from(id)
                .map(Some)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            let alignment_start = slice_header.alignment_start();
            let alignment_span = slice_header.alignment_span();

            (reference_sequence_id, alignment_start, alignment_span)
        }
        ReferenceSequenceId::None => (None, None, 0),
        ReferenceSequenceId::Many => unreachable!(),
    };

    let record = crai::Record::new(
        reference_sequence_id,
        alignment_start,
        alignment_span,
        container_position,
        landmark,
        slice_length,
    );

    index.push(record);

    Ok(())
}
