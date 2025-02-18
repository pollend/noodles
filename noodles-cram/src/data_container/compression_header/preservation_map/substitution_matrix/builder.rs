use super::{Histogram, SubstitutionMatrix};
use crate::{
    record::{feature::substitution, Feature},
    Record,
};

#[derive(Debug, Default)]
pub struct Builder {
    histogram: Histogram,
}

impl Builder {
    pub fn update(&mut self, record: &Record) {
        for feature in record.features().iter() {
            match feature {
                Feature::Substitution(_, substitution::Value::Bases(reference_base, read_base)) => {
                    self.histogram.hit(*reference_base, *read_base);
                }
                Feature::Substitution(_, substitution::Value::Code(_)) => {
                    panic!("substitution matrix cannot be built from substitution codes");
                }
                _ => {}
            }
        }
    }

    pub fn build(self) -> SubstitutionMatrix {
        SubstitutionMatrix::from(self.histogram)
    }
}

#[cfg(test)]
mod tests {
    use noodles_core::Position;
    use noodles_sam as sam;

    use super::*;

    #[test]
    fn test_build() -> Result<(), Box<dyn std::error::Error>> {
        use sam::record::Sequence;

        use crate::record::feature::substitution::{self, Base};

        // reference sequence = "ACAGGAATAANNNNNN"
        let bases: Sequence = "TCTGGCGTGT".parse()?;

        let record = Record::builder()
            .set_alignment_start(Position::try_from(1)?)
            .set_read_length(bases.len())
            .set_bases(bases)
            .add_feature(Feature::Substitution(
                Position::try_from(1)?,
                substitution::Value::Bases(Base::A, Base::T),
            ))
            .add_feature(Feature::Substitution(
                Position::try_from(3)?,
                substitution::Value::Bases(Base::A, Base::T),
            ))
            .add_feature(Feature::Substitution(
                Position::try_from(6)?,
                substitution::Value::Bases(Base::A, Base::C),
            ))
            .add_feature(Feature::Substitution(
                Position::try_from(7)?,
                substitution::Value::Bases(Base::A, Base::G),
            ))
            .add_feature(Feature::Substitution(
                Position::try_from(9)?,
                substitution::Value::Bases(Base::A, Base::G),
            ))
            .add_feature(Feature::Substitution(
                Position::try_from(10)?,
                substitution::Value::Bases(Base::A, Base::T),
            ))
            .build();

        let mut builder = Builder::default();
        builder.update(&record);
        let matrix = builder.build();

        assert_eq!(
            matrix.substitutions,
            [
                [Base::T, Base::G, Base::C, Base::N],
                [Base::A, Base::G, Base::T, Base::N],
                [Base::A, Base::C, Base::T, Base::N],
                [Base::A, Base::C, Base::G, Base::N],
                [Base::A, Base::C, Base::G, Base::T],
            ]
        );

        Ok(())
    }
}
