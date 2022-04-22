//
// Copyright 2022 Garry Xu
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_till1},
    character::complete::multispace0,
    character::complete::one_of,
    combinator::map,
    error::ParseError,
    multi::separated_list0,
    number::complete::double,
    sequence::{delimited, separated_pair},
    Err as NomErr, IResult,
};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::str::FromStr;

type Result<I, O> = IResult<I, O, Error>;

/// Errors associated with parsing JSON strings or files.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Errors generated during parsing string slice to a specific JSON value.
    /// More informance can be retrieved from the ErrorKind field.
    ParsingError(ErrorKind),
    /// Errors generated during file operations.
    IOError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match self {
            ParsingError(kind) => write!(f, "{}", kind),
            IOError(s) => write!(
                f,
                "An I/O error occurs during loading the JSON file, error={}.",
                s
            ),
        }
    }
}

impl<I> ParseError<I> for Error {
    // Convert a nom::error::Error to a Error::ParseError
    fn from_error_kind(_: I, _: nom::error::ErrorKind) -> Self {
        Self::ParsingError(ErrorKind::Other)
    }

    fn append(_: I, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl From<std::io::Error> for Error {
    // Convert a io::Error to json::Error
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    InvalidNull,
    InvalidBoolean,
    InvalidNumber,
    InvalidString,
    InvalidArray,
    InvalidObject,
    Other,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ErrorKind::*;

        match self {
            InvalidNull => write!(
                f,
                "The given input can't be parsed into a valid Null value."
            ),
            InvalidBoolean => write!(
                f,
                "The given input can't be parsed into a valid true/false value."
            ),
            InvalidNumber => write!(f, "The given input can't be parsed into a valid number"),
            InvalidString => write!(f, "The given input can't be parsed into a valid string"),
            InvalidArray => write!(f, "The given input can't be parsed into a valid JSON array"),
            InvalidObject => write!(
                f,
                "The given input can't be parsed into a valid JSON object"
            ),
            _ => write!(f, "An internal error occurs."),
        }
    }
}

/// A simplified abstraction for a JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
}

impl FromStr for Json {
    type Err = Error;

    // Generate a json object from the string slice.
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(
            delimited(
                multispace0, 
                Self::object, 
                multispace0
            )(s).map_err(
                |e: NomErr<Error>| match e {
                    // Unwrap the nom::error::Error
                    NomErr::Error(v) => v,
                    _ => Error::ParsingError(ErrorKind::InvalidObject)
                }
            )?.1
        )
    }
}

/// Map a nom error to corresponding ErrorKind
macro_rules! map_err {
    ($result:expr, $kind:expr) => {
        $result.map_err(|e: NomErr<Error>| match e {
            NomErr::Error(Error::ParsingError(ErrorKind::Other)) => {
                NomErr::Error(Error::ParsingError($kind))
            }
            _ => e,
        })
    };
}

impl Json {
    /// Generate a json object by reading provided file.
    pub fn from_file(path: &str) -> std::result::Result<Self, Error> {
        Ok(fs::read_to_string(path)?.parse::<Json>()?)
    }

    /// Take a JSON value from a JSON object.
    fn take(&mut self, key: &str) -> Option<Json> {
        match self {
            Json::Object(map) => map.remove(key),
            _ => None
        }
    }
    /// Take a JSON value and convert it into a number.
    pub fn take_number(&mut self, key: &str) -> Option<f64> {
        match self.take(key) {
            Some(Json::Number(v)) => Some(v),
            _ => None
        }
    }

    /// Take a JSON value and convert it into a string.
    pub fn take_string(&mut self, key: &str) -> Option<String> {
        match self.take(key) {
            Some(Json::String(s)) => Some(s),
            _ => None
        }
    }

    /// Take a JSON object value.
    pub fn take_object(&mut self, key: &str) -> Option<Json> {
        match self.take(key) {
            Some(Json::Object(v)) => Some(Json::Object(v)),
            _ => None
        }
    }

    /// Take a JSON array value.
    pub fn take_array(&mut self, key: &str) -> Option<Vec<Json>> {
        match self.take(key) {
            Some(Json::Array(v)) => Some(v),
            _ => None
        }
    }

    /// Parse a JSON null value
    fn null(s: &str) -> Result<&str, Json> {
        map_err!(map(tag("null"), |_| Json::Null)(s), ErrorKind::InvalidNull)
    }

    /// Parse a JSON boolean value(true or false).
    fn boolean(s: &str) -> Result<&str, Json> {
        map_err!(
            alt((
                map(tag("true"), |_| Json::Boolean(true)),
                map(tag("false"), |_| Json::Boolean(false)),
            ))(s),
            ErrorKind::InvalidBoolean
        )
    }

    /// Parse a JSON number.
    fn number(s: &str) -> Result<&str, Json> {
        map_err!(
            double(s).map(|(s, v)| (s, Json::Number(v))),
            ErrorKind::InvalidNumber
        )
    }

    /// Parse a JSON string value.
    fn string(s: &str) -> Result<&str, &str> {
        map_err!(
            delimited(
                tag("\""),
                escaped(
                    take_till1(|c: char| c == '\\' || c == '\"' || c.is_ascii_control()),
                    '\\',
                    one_of(r#""\/bfnrtu"#),
                ),
                tag("\""),
            )(s),
            ErrorKind::InvalidString
        )
    }

    /// Parse a JSON array.
    fn array(s: &str) -> Result<&str, Json> {
        map_err!(
            delimited(
                tag("["),
                separated_list0(tag(","), delimited(multispace0, Self::value, multispace0)),
                tag("]"),
            )(s)
            .map(|(s, v)| (s, Json::Array(v))),
            ErrorKind::InvalidArray
        )
    }

    /// Parse a JSON object.
    fn object(s: &str) -> Result<&str, Json> {
        map_err!(
            map(
                delimited(
                    tag("{"),
                    separated_list0(
                        tag(","),
                        delimited(
                            multispace0,
                            separated_pair(
                                delimited(
                                    multispace0, 
                                    Self::string, 
                                    multispace0
                                ),
                                tag(":"),
                                delimited(
                                    multispace0, 
                                    Self::value, 
                                    multispace0
                                ),
                            ),
                            multispace0,
                        ),
                    ),
                    tag("}"),
                ),
                |vec: Vec<(&str, Json)>| {
                    Json::Object(
                        vec.into_iter().map(
                            |(s, v)| (s.to_string(), v)
                        ).collect()
                    )
                },
            )(s),
            ErrorKind::InvalidObject
        )
    }

    /// Parse a JSON value
    fn value(s: &str) -> Result<&str, Json> {
        alt((
            Self::null,
            Self::boolean,
            Self::number,
            map(Self::string, |v: &str| Json::String(v.to_string())),
            Self::array,
            Self::object,
        ))(s)
    }
}
#[test]
pub fn test_null() {
    assert_eq!(Json::null("null"), Ok(("", Json::Null)));
    assert_eq!(
        Json::null("NULL"),
        Err(NomErr::Error(Error::ParsingError(ErrorKind::InvalidNull)))
    );
}

#[test]
pub fn test_boolean() {
    assert_eq!(Json::boolean("true"), Ok(("", Json::Boolean(true))));
    assert_eq!(Json::boolean("false"), Ok(("", Json::Boolean(false))));
    assert_eq!(
        Json::boolean("True"),
        Err(NomErr::Error(Error::ParsingError(
            ErrorKind::InvalidBoolean
        )))
    );
}

#[test]
pub fn test_number() {
    assert_eq!(Json::number("2.0"), Ok(("", Json::Number(2.0f64))));
    assert_eq!(Json::number("2.#"), Ok(("#", Json::Number(2.0f64))));
    assert_eq!(
        Json::number("a2"),
        Err(NomErr::Error(Error::ParsingError(ErrorKind::InvalidNumber)))
    );
}

#[test]
pub fn test_string() {
    assert_eq!(Json::string(r#""abc""#), Ok(("", r#"abc"#)));
    assert_eq!(Json::string(r#""abc02""#), Ok(("", r#"abc02"#)));
    assert_eq!(Json::string(r#""abc_def""#), Ok(("", r#"abc_def"#)));
    assert_eq!(Json::string(r#""abc_\bef""#), Ok(("", r#"abc_\bef"#)));
    assert_eq!(Json::string(r#""abc_\nef""#), Ok(("", r#"abc_\nef"#)));
    assert_eq!(Json::string(r#""abc_\"ef""#), Ok(("", r#"abc_\"ef"#)));
    assert_eq!(Json::string(r#""a""#), Ok(("", r#"a"#)));
    assert_eq!(
        Json::string(r#""""#),
        Err(NomErr::Error(Error::ParsingError(ErrorKind::InvalidString)))
    );
    assert_eq!(
        Json::string(r#"abc"#),
        Err(NomErr::Error(Error::ParsingError(ErrorKind::InvalidString)))
    );
}

#[test]
pub fn test_array() {
    assert_eq!(
        Json::array(r#"["abc", 234, true, null]"#),
        Ok((
            "",
            Json::Array(vec![
                Json::String("abc".to_string()),
                Json::Number(234f64),
                Json::Boolean(true),
                Json::Null
            ])
        )),
    );
    assert_eq!(
        Json::array(r#"[ "abc" ]"#),
        Ok(("", Json::Array(vec![Json::String("abc".to_string())]))),
    );
    assert_eq!(
        Json::array(r#"["abc", 234, true, null, "bug":"bug"]"#),
        Err(NomErr::Error(Error::ParsingError(ErrorKind::InvalidArray))),
    );
}

#[test]
pub fn test_object() {
    let ok_data = r#"{  "null": null, "bool": true, "num": 456, 
    "str": "test_string", "arr": ["abc", 234, true, null],
    "object": {"str": "value"} }"#;
    let mut map = HashMap::new();
    map.insert("null".to_string(), Json::Null);
    map.insert("bool".to_string(), Json::Boolean(true));
    map.insert("num".to_string(), Json::Number(456f64));
    map.insert("str".to_string(), Json::String("test_string".to_string()));
    map.insert(
        "arr".to_string(),
        Json::Array(vec![
            Json::String("abc".to_string()),
            Json::Number(234f64),
            Json::Boolean(true),
            Json::Null,
        ]),
    );
    let mut sub_map = HashMap::new();
    sub_map.insert("str".to_string(), Json::String("value".to_string()));
    map.insert("object".to_string(), Json::Object(sub_map));

    assert_eq!(Json::object(ok_data), Ok(("", Json::Object(map))),);
}

#[test]
pub fn test_from_file() {
    macro_rules! hashmap {
        ( $( $key: expr => $val: expr ),* ) => {{
             let mut map: HashMap<String, Json> = HashMap::new();
             $( map.insert($key, $val); )*
             map
        }}
    }

    let object = Json::from_file("../../resources/vm-example.json");

    let data = Json::Object(hashmap![
        "cpu".to_string() => Json::Object(
            hashmap!["count".to_string() => Json::Number(2f64)]
        ),
        "memory".to_string() => Json::Object(
            hashmap!["size_mib".to_string() => Json::Number(1024f64)]
        ),
        "device".to_string() => Json::Array(
            vec![
                Json::Object(
                    hashmap![
                        "driver".to_string() => {
                            Json::String("virtio-blk".to_string())
                        },
                        "source".to_string() => {
                            Json::String(
                                "focal-server-cloudimg-amd64.raw".to_string()
                            )
                        }
                    ]
                ),
                Json::Object(
                    hashmap![
                        "driver".to_string() => {
                            Json::String("virtio-net".to_string())
                        },
                        "mac".to_string() => {
                            Json::String("fa:16:3e:21:c0:c0".to_string())
                        }
                    ]
                ),
                Json::Object(
                    hashmap![
                        "driver".to_string() => {
                            Json::String("vfio".to_string())
                        },
                        "source".to_string() => {
                            Json::String("02:00.0".to_string())
                        }
                    ]
                ),
                Json::Object(
                    hashmap![
                        "driver".to_string() => {
                            Json::String("console".to_string())
                        },
                        "type".to_string() => Json::String("tty".to_string())
                    ]
                ),
            ]
        ),
        "os".to_string() => Json::Object(
            hashmap![
                    "kernel".to_string() => {
                        Json::String("/tmp/test-vm/vmlinux.bin".to_string())
                    },
                    "initrd".to_string() => Json::Null,
                    "rootfs".to_string() => {
                        Json::String(
                            "/tmp/test-vm/bionic.rootfs.ext4".to_string()
                        )
                    },
                    "cmdline".to_string() => {
                        Json::String(
                            "console=ttyS0 reboot=k panic=1 pci=off".to_string()
                        )
                    }
            ]
        ),
        "vmm".to_string() => Json::Object(hashmap![])
    ]);
    assert_eq!(object, Ok(data));
}
