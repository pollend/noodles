use std::{io, mem};

use noodles_fasta as fasta;
use noodles_sam as sam;

use super::{slice, CompressionHeader, DataContainer, Slice};
use crate::{writer::Options, Record};

const MAX_SLICE_COUNT: usize = 1;

#[derive(Debug)]
pub struct Builder {
    slice_builder: slice::Builder,
    slice_builders: Vec<slice::Builder>,
    record_counter: i64,
    base_count: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AddRecordError {
    ContainerFull(Record),
    SliceFull(Record),
}

impl Builder {
    pub fn new(record_counter: i64) -> Self {
        Self {
            slice_builder: Slice::builder(),
            slice_builders: Vec::new(),
            record_counter,
            base_count: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.slice_builder.is_empty() && self.slice_builders.is_empty()
    }

    pub fn base_count(&self) -> i64 {
        self.base_count
    }

    pub fn add_record(&mut self, record: Record) -> Result<(), AddRecordError> {
        if self.slice_builders.len() >= MAX_SLICE_COUNT {
            return Err(AddRecordError::ContainerFull(record));
        }

        match self.slice_builder.add_record(record) {
            Ok(r) => {
                self.base_count += r.read_length() as i64;
                Ok(())
            }
            Err(e) => match e {
                slice::builder::AddRecordError::SliceFull(r) => {
                    let slice_builder = mem::take(&mut self.slice_builder);
                    self.slice_builders.push(slice_builder);
                    Err(AddRecordError::SliceFull(r))
                }
                slice::builder::AddRecordError::ReferenceSequenceIdMismatch(r) => {
                    Err(AddRecordError::ContainerFull(r))
                }
            },
        }
    }

    pub fn build(
        mut self,
        options: &Options,
        reference_sequence_repository: &fasta::Repository,
        header: &sam::Header,
    ) -> io::Result<DataContainer> {
        if !self.slice_builder.is_empty() {
            self.slice_builders.push(self.slice_builder);
        }

        let compression_header = build_compression_header(options, &self.slice_builders);

        let record_counter = self.record_counter;
        let slices = self
            .slice_builders
            .into_iter()
            .map(|builder| {
                builder.build(
                    reference_sequence_repository,
                    header,
                    &compression_header,
                    record_counter,
                )
            })
            .collect::<Result<_, _>>()?;

        Ok(DataContainer {
            compression_header,
            slices,
        })
    }
}

fn build_compression_header(
    options: &Options,
    slice_builders: &[slice::Builder],
) -> CompressionHeader {
    let mut compression_header_builder = CompressionHeader::builder();
    compression_header_builder.apply_options(options);

    for slice_builder in slice_builders {
        for record in slice_builder.records() {
            compression_header_builder.update(record);
        }
    }

    compression_header_builder.build()
}
