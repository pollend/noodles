//! Replaces the header of a VCF file.
//!
//! This is similar to the functionality of `bcftools reheader --header <header-src> <src>`.
//!
//! Verify the output by piping to `bcftools view --no-version --header`.

use std::env;

use futures::TryStreamExt;
use noodles_vcf as vcf;
use tokio::{
    fs::File,
    io::{self, BufReader},
};

fn add_comment(header: &mut vcf::Header) {
    use vcf::header::{
        record::{Key, Value},
        Record,
    };

    header.insert(Record::new(
        Key::Other(String::from("comment")),
        Value::String(String::from("a comment added by noodles-vcf")),
    ));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src = env::args().nth(1).expect("missing src");

    let mut reader = File::open(src)
        .await
        .map(BufReader::new)
        .map(vcf::AsyncReader::new)?;

    let mut header = reader.read_header().await?.parse()?;
    add_comment(&mut header);

    let mut writer = vcf::AsyncWriter::new(io::stdout());
    writer.write_header(&header).await?;

    let mut records = reader.records(&header);

    while let Some(record) = records.try_next().await? {
        writer.write_record(&record).await?;
    }

    Ok(())
}
