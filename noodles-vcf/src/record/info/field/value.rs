//! VCF record info field value.

use std::{error, fmt, num, str};

use super::MISSING_VALUE;
use crate::{
    header::{info::Type, Info, Number},
    record::value::{self, percent_decode},
};

const DELIMITER: char = ',';

/// A VCF record info field value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// An 32-bit integer.
    Integer(i32),
    /// A single-precision floating-point.
    Float(f32),
    /// A boolean.
    Flag,
    /// A character.
    Character(char),
    /// A string.
    String(String),
    /// An array of 32-bit integers.
    IntegerArray(Vec<Option<i32>>),
    /// An array of single-precision floating-points.
    FloatArray(Vec<Option<f32>>),
    /// An array of characters.
    CharacterArray(Vec<Option<char>>),
    /// An array of strings.
    StringArray(Vec<Option<String>>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(n) => write!(f, "{}", n),
            Self::Float(n) => write!(f, "{}", n),
            Self::Flag => Ok(()),
            Self::Character(c) => write!(f, "{}", c),
            Self::String(s) => write!(f, "{}", s),
            Self::IntegerArray(values) => {
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, "{}", DELIMITER)?;
                    }

                    if let Some(v) = value {
                        write!(f, "{}", v)?;
                    } else {
                        f.write_str(MISSING_VALUE)?;
                    }
                }

                Ok(())
            }
            Self::FloatArray(values) => {
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, "{}", DELIMITER)?;
                    }

                    if let Some(v) = value {
                        write!(f, "{}", v)?;
                    } else {
                        f.write_str(MISSING_VALUE)?;
                    }
                }

                Ok(())
            }
            Self::CharacterArray(values) => {
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, "{}", DELIMITER)?;
                    }

                    if let Some(v) = value {
                        write!(f, "{}", v)?;
                    } else {
                        f.write_str(MISSING_VALUE)?;
                    }
                }

                Ok(())
            }
            Self::StringArray(values) => {
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, "{}", DELIMITER)?;
                    }

                    if let Some(v) = value {
                        write!(f, "{}", v)?;
                    } else {
                        f.write_str(MISSING_VALUE)?;
                    }
                }

                Ok(())
            }
        }
    }
}

/// An error returned when a raw VCF record info field value fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The field cardinality is invalid for the type.
    InvalidNumberForType(Number, Type),
    /// The integer is invalid.
    InvalidInteger(num::ParseIntError),
    /// The floating-point is invalid.
    InvalidFloat(num::ParseFloatError),
    /// The flag is invalid.
    InvalidFlag,
    /// The character is invalid.
    InvalidCharacter,
    /// The string is invalid.
    InvalidString(str::Utf8Error),
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNumberForType(number, ty) => {
                write!(f, "invalid number {:?} for type {:?}", number, ty)
            }
            Self::InvalidInteger(e) => write!(f, "invalid integer: {}", e),
            Self::InvalidFloat(e) => write!(f, "invalid float: {}", e),
            Self::InvalidFlag => f.write_str("invalid flag"),
            Self::InvalidCharacter => f.write_str("invalid character"),
            Self::InvalidString(e) => write!(f, "invalid string: {}", e),
        }
    }
}

impl Value {
    /// Parses a raw info field value with the given info header record.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::{header::{info::Key, Info}, record::info::field::Value};
    /// let info = Info::from(Key::SamplesWithDataCount);
    /// assert_eq!(Value::from_str_info("1", &info), Ok(Value::Integer(1)));
    /// ```
    pub fn from_str_info(s: &str, info: &Info) -> Result<Self, ParseError> {
        match info.ty() {
            Type::Integer => match info.number() {
                Number::Count(0) => Err(ParseError::InvalidNumberForType(info.number(), info.ty())),
                Number::Count(1) => parse_i32(s),
                _ => parse_i32_array(s),
            },
            Type::Float => match info.number() {
                Number::Count(0) => Err(ParseError::InvalidNumberForType(info.number(), info.ty())),
                Number::Count(1) => parse_f32(s),
                _ => parse_f32_array(s),
            },
            Type::Flag => match info.number() {
                Number::Count(0) => parse_flag(s),
                _ => Err(ParseError::InvalidNumberForType(info.number(), info.ty())),
            },
            Type::Character => match info.number() {
                Number::Count(0) => Err(ParseError::InvalidNumberForType(info.number(), info.ty())),
                Number::Count(1) => parse_char(s),
                _ => parse_char_array(s),
            },
            Type::String => match info.number() {
                Number::Count(0) => Err(ParseError::InvalidNumberForType(info.number(), info.ty())),
                Number::Count(1) => parse_string(s),
                _ => parse_string_array(s),
            },
        }
    }
}

fn parse_i32(s: &str) -> Result<Value, ParseError> {
    s.parse()
        .map(Value::Integer)
        .map_err(ParseError::InvalidInteger)
}

fn parse_i32_array(s: &str) -> Result<Value, ParseError> {
    s.split(DELIMITER)
        .map(|t| match t {
            MISSING_VALUE => Ok(None),
            _ => t.parse().map(Some).map_err(ParseError::InvalidInteger),
        })
        .collect::<Result<_, _>>()
        .map(Value::IntegerArray)
}

fn parse_f32(s: &str) -> Result<Value, ParseError> {
    value::parse_f32(s)
        .map(Value::Float)
        .map_err(ParseError::InvalidFloat)
}

fn parse_f32_array(s: &str) -> Result<Value, ParseError> {
    s.split(DELIMITER)
        .map(|t| match t {
            MISSING_VALUE => Ok(None),
            _ => value::parse_f32(t)
                .map(Some)
                .map_err(ParseError::InvalidFloat),
        })
        .collect::<Result<_, _>>()
        .map(Value::FloatArray)
}

fn parse_flag(s: &str) -> Result<Value, ParseError> {
    if s.is_empty() {
        Ok(Value::Flag)
    } else {
        Err(ParseError::InvalidFlag)
    }
}

fn parse_raw_char(s: &str) -> Result<char, ParseError> {
    let mut chars = s.chars();

    if let Some(c) = chars.next() {
        if chars.next().is_none() {
            return Ok(c);
        }
    }

    Err(ParseError::InvalidCharacter)
}

fn parse_char(s: &str) -> Result<Value, ParseError> {
    parse_raw_char(s).map(Value::Character)
}

fn parse_char_array(s: &str) -> Result<Value, ParseError> {
    s.split(DELIMITER)
        .map(|t| match t {
            MISSING_VALUE => Ok(None),
            _ => parse_raw_char(t).map(Some),
        })
        .collect::<Result<_, _>>()
        .map(Value::CharacterArray)
}

fn parse_string(s: &str) -> Result<Value, ParseError> {
    percent_decode(s)
        .map(|t| Value::String(t.into()))
        .map_err(ParseError::InvalidString)
}

fn parse_string_array(s: &str) -> Result<Value, ParseError> {
    s.split(DELIMITER)
        .map(|t| match t {
            MISSING_VALUE => Ok(None),
            _ => percent_decode(t)
                .map(|u| Some(u.into()))
                .map_err(ParseError::InvalidString),
        })
        .collect::<Result<_, _>>()
        .map(Value::StringArray)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt() {
        let value = Value::Integer(2);
        assert_eq!(value.to_string(), "2");

        let value = Value::Float(0.333);
        assert_eq!(value.to_string(), "0.333");

        assert_eq!(Value::Flag.to_string(), "");

        let value = Value::Character('n');
        assert_eq!(value.to_string(), "n");

        let value = Value::String(String::from("noodles"));
        assert_eq!(value.to_string(), "noodles");

        let value = Value::IntegerArray(vec![Some(2)]);
        assert_eq!(value.to_string(), "2");

        let value = Value::IntegerArray(vec![Some(2), Some(5)]);
        assert_eq!(value.to_string(), "2,5");

        let value = Value::IntegerArray(vec![Some(2), None]);
        assert_eq!(value.to_string(), "2,.");

        let value = Value::FloatArray(vec![Some(0.333)]);
        assert_eq!(value.to_string(), "0.333");

        let value = Value::FloatArray(vec![Some(0.333), Some(0.667)]);
        assert_eq!(value.to_string(), "0.333,0.667");

        let value = Value::FloatArray(vec![Some(0.333), None]);
        assert_eq!(value.to_string(), "0.333,.");

        let value = Value::CharacterArray(vec![Some('n')]);
        assert_eq!(value.to_string(), "n");

        let value = Value::CharacterArray(vec![Some('n'), Some('d'), Some('l'), Some('s')]);
        assert_eq!(value.to_string(), "n,d,l,s");

        let value = Value::CharacterArray(vec![Some('n'), Some('d'), Some('l'), None]);
        assert_eq!(value.to_string(), "n,d,l,.");

        let value = Value::StringArray(vec![Some(String::from("noodles"))]);
        assert_eq!(value.to_string(), "noodles");

        let value = Value::StringArray(vec![
            Some(String::from("noodles")),
            Some(String::from("vcf")),
        ]);
        assert_eq!(value.to_string(), "noodles,vcf");

        let value = Value::StringArray(vec![Some(String::from("noodles")), None]);
        assert_eq!(value.to_string(), "noodles,.");
    }

    #[test]
    fn test_from_str_info_with_integer() -> Result<(), crate::header::info::key::ParseError> {
        let info = Info::new(
            "I32".parse()?,
            Number::Count(0),
            Type::Integer,
            String::default(),
        );
        assert_eq!(
            Value::from_str_info("8", &info),
            Err(ParseError::InvalidNumberForType(
                Number::Count(0),
                Type::Integer
            ))
        );

        let info = Info::new(
            "I32".parse()?,
            Number::Count(1),
            Type::Integer,
            String::default(),
        );
        assert_eq!(Value::from_str_info("8", &info), Ok(Value::Integer(8)));

        let info = Info::new(
            "I32".parse()?,
            Number::Count(2),
            Type::Integer,
            String::default(),
        );
        assert_eq!(
            Value::from_str_info("8,13", &info),
            Ok(Value::IntegerArray(vec![Some(8), Some(13)])),
        );
        assert_eq!(
            Value::from_str_info("8,.", &info),
            Ok(Value::IntegerArray(vec![Some(8), None])),
        );

        Ok(())
    }

    #[test]
    fn test_from_str_info_with_float() -> Result<(), crate::header::info::key::ParseError> {
        let info = Info::new(
            "F32".parse()?,
            Number::Count(0),
            Type::Float,
            String::default(),
        );
        assert_eq!(
            Value::from_str_info("0.333", &info),
            Err(ParseError::InvalidNumberForType(
                Number::Count(0),
                Type::Float
            ))
        );

        let info = Info::new(
            "F32".parse()?,
            Number::Count(1),
            Type::Float,
            String::default(),
        );
        assert_eq!(
            Value::from_str_info("0.333", &info),
            Ok(Value::Float(0.333))
        );

        let info = Info::new(
            "F32".parse()?,
            Number::Count(2),
            Type::Float,
            String::default(),
        );
        assert_eq!(
            Value::from_str_info("0.333,0.667", &info),
            Ok(Value::FloatArray(vec![Some(0.333), Some(0.667)]))
        );
        assert_eq!(
            Value::from_str_info("0.333,.", &info),
            Ok(Value::FloatArray(vec![Some(0.333), None]))
        );

        Ok(())
    }

    #[test]
    fn test_from_str_info_with_flag() -> Result<(), crate::header::info::key::ParseError> {
        let info = Info::new(
            "BOOL".parse()?,
            Number::Count(0),
            Type::Flag,
            String::default(),
        );
        assert_eq!(Value::from_str_info("", &info), Ok(Value::Flag));

        let info = Info::new(
            "BOOL".parse()?,
            Number::Count(0),
            Type::Flag,
            String::default(),
        );
        assert_eq!(
            Value::from_str_info("true", &info),
            Err(ParseError::InvalidFlag)
        );

        let info = Info::new(
            "BOOL".parse()?,
            Number::Count(1),
            Type::Flag,
            String::default(),
        );
        assert_eq!(
            Value::from_str_info("", &info),
            Err(ParseError::InvalidNumberForType(
                Number::Count(1),
                Type::Flag
            ))
        );

        Ok(())
    }

    #[test]
    fn test_from_str_info_with_character() -> Result<(), crate::header::info::key::ParseError> {
        let info = Info::new(
            "CHAR".parse()?,
            Number::Count(0),
            Type::Character,
            String::default(),
        );
        assert_eq!(
            Value::from_str_info("n", &info),
            Err(ParseError::InvalidNumberForType(
                Number::Count(0),
                Type::Character
            ))
        );

        let info = Info::new(
            "CHAR".parse()?,
            Number::Count(1),
            Type::Character,
            String::default(),
        );
        assert_eq!(Value::from_str_info("n", &info), Ok(Value::Character('n')));

        let info = Info::new(
            "CHAR".parse()?,
            Number::Count(2),
            Type::Character,
            String::default(),
        );
        assert_eq!(
            Value::from_str_info("n,d,l,s", &info),
            Ok(Value::CharacterArray(vec![
                Some('n'),
                Some('d'),
                Some('l'),
                Some('s')
            ]))
        );
        assert_eq!(
            Value::from_str_info("n,d,l,.", &info),
            Ok(Value::CharacterArray(vec![
                Some('n'),
                Some('d'),
                Some('l'),
                None
            ]))
        );

        Ok(())
    }

    #[test]
    fn test_from_str_info_with_string() -> Result<(), crate::header::info::key::ParseError> {
        let info = Info::new(
            "STRING".parse()?,
            Number::Count(0),
            Type::String,
            String::new(),
        );
        assert_eq!(
            Value::from_str_info("noodles", &info),
            Err(ParseError::InvalidNumberForType(
                Number::Count(0),
                Type::String
            ))
        );

        let info = Info::new(
            "STRING".parse()?,
            Number::Count(1),
            Type::String,
            String::new(),
        );
        assert_eq!(
            Value::from_str_info("noodles", &info),
            Ok(Value::String(String::from("noodles")))
        );
        assert_eq!(
            Value::from_str_info("8%25", &info),
            Ok(Value::String(String::from("8%")))
        );

        let info = Info::new(
            "STRING".parse()?,
            Number::Count(2),
            Type::String,
            String::new(),
        );
        assert_eq!(
            Value::from_str_info("noodles,vcf", &info),
            Ok(Value::StringArray(vec![
                Some(String::from("noodles")),
                Some(String::from("vcf"))
            ]))
        );
        assert_eq!(
            Value::from_str_info("noodles,.", &info),
            Ok(Value::StringArray(vec![
                Some(String::from("noodles")),
                None
            ]))
        );
        assert_eq!(
            Value::from_str_info("8%25,13%25", &info),
            Ok(Value::StringArray(vec![
                Some(String::from("8%")),
                Some(String::from("13%"))
            ]))
        );

        Ok(())
    }
}
