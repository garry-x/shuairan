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

use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use nom::{
	branch::alt,
	bytes::complete::{escaped, tag, take_till1},
	character::complete::one_of,
	combinator::map,
	error::{ErrorKind, ParseError},
	multi::separated_list0,
    character::complete::multispace0,
	number::complete::double,
	sequence::{delimited, separated_pair},
	Err as NomErr, IResult,
};

/// Errors associated with parsing JSON strings.
#[derive(Debug, PartialEq)]
pub enum JsonError {
    /// The string slice is not a valid JSON "null" value.
    InvalidNull,
    /// The string slice is not a valid JSON "true/false" value.
    InvalidBoolean,
    /// The string slice is not a valid JSON number.
    InvalidNumber,
    /// The string slice is not a valid JSON string.
    InvalidString,
    /// The string slice is not a valid JSON array.
    InvalidArray,
    /// The string slice is not a valid JSON object.
    InvalidObject,
    /// Errors associated with file operations 
    IOError(String),
    /// Errors generated by the nom parser.
    NomError(ErrorKind),
}

type JResult<I, O> = IResult<I, O, JsonError>;

impl<I> ParseError<I> for JsonError {
 	fn from_error_kind(_: I, kind: ErrorKind) -> Self {
		Self::NomError(kind)
  	}

  	fn append(_: I, _: ErrorKind, other: Self) -> Self {
    	other
  	}
}

impl From<std::io::Error> for JsonError {
    fn from(e: std::io::Error) -> Self {
        JsonError::IOError(e.to_string())
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
    Object(HashMap<String, Json>) 
}

impl FromStr for Json {
    type Err = JsonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(object(s).map_err(
            |e: NomErr<JsonError>| {
                match e {
                    NomErr::Error(v) => v,
                    _ => JsonError::InvalidObject, 
                }
            }
        )?.1) 
    }
}

impl Json {
    pub fn from_file(path: &str) -> Result<Self, JsonError> {
        Ok(fs::read_to_string(path)?.parse::<Json>()?)
    }
}

/// Match a null value
fn null(s: &str) -> JResult<&str, Json> {
    map(tag("null"), |_| Json::Null)(s).map_err(
        |_: NomErr<JsonError>| NomErr::Error(JsonError::InvalidNull)
    )
}

/// Match an boolean value.
fn boolean(s: &str) -> JResult<&str, Json> {
    alt((
        map(tag("true"), |_| Json::Boolean(true)),
        map(tag("false"), |_| Json::Boolean(false)),
    ))(s).map_err(
        |_: NomErr<JsonError>| NomErr::Error(JsonError::InvalidBoolean)
    )
}

/// Match a f64 value. 
fn number(s: &str) -> JResult<&str, Json> {
    double(s).map(|(s, v)| (s, Json::Number(v))).map_err(
        |_: NomErr<JsonError>| NomErr::Error(JsonError::InvalidNumber)
    ) 
}

/// Match a string value.
fn string(s: &str) -> JResult<&str, &str> {
    delimited(
        tag("\""),
        escaped(
            take_till1(|c: char| c == '\\' || c == '\"' || c.is_ascii_control()),
            '\\',
            one_of(r#""\/bfnrtu"#),
        ),
        tag("\""),
    )(s).map_err(|_: NomErr<JsonError>| NomErr::Error(JsonError::InvalidString))
}

/// Match a JSON array.
fn array(s: &str) -> JResult<&str, Json> {
	delimited(
        tag("["),
        separated_list0(tag(","), delimited(multispace0, value, multispace0)), 
        tag("]"),
    )(s).map(|(s, v)| (s, Json::Array(v))).map_err(
        |e: NomErr<JsonError>| {
            match e {
                NomErr::Error(JsonError::NomError(_)) =>
                    NomErr::Error(JsonError::InvalidArray),
                other => other,
            }
        }
    )
}

/// Match a JSON object.
fn object(s: &str) -> JResult<&str, Json> {
    map(
        delimited(
            multispace0,
            delimited(
                tag("{"),
                separated_list0(
                    tag(","), 
                    delimited(
                        multispace0,
                        separated_pair(
                            delimited(multispace0, string, multispace0),
                            tag(":"),
                            delimited(multispace0, value, multispace0),
                        ),
                        multispace0,
                    ),
                ),
                tag("}"),
            ),
            multispace0,
        ),
        |vec: Vec<(&str, Json)>| {
            Json::Object(
                vec.into_iter().map(|(s, v)| (s.to_string(), v)).collect()
            )
        },
    )(s).map_err(
        |e: NomErr<JsonError>| {
            match e {
                NomErr::Error(JsonError::NomError(_)) => 
                    NomErr::Error(JsonError::InvalidObject),
                other => other,
            }
        }
    )
}

/// Match a JSON value
fn value(s: &str) -> JResult<&str, Json> {
	alt((
        null,
        boolean,
        number,
        map(string, |v: &str| Json::String(v.to_string())),
        array,
        object,
    ))(s).map_err(
        |e: NomErr<JsonError>| {
            match e {
                NomErr::Error(JsonError::NomError(_)) =>
                    NomErr::Error(JsonError::InvalidArray),
                other => other,
            }
        }
    )
}

#[test]
pub fn test_null() {
    assert_eq!(null("null"), Ok(("", Json::Null)));
    assert_eq!(null("NULL"), Err(NomErr::Error(JsonError::InvalidNull)));
}

#[test]
pub fn test_boolean() {
    assert_eq!(boolean("true"), Ok(("", Json::Boolean(true))));
    assert_eq!(boolean("false"), Ok(("", Json::Boolean(false))));
    assert_eq!(boolean("True"), Err(NomErr::Error(JsonError::InvalidBoolean)));
}

#[test]
pub fn test_number() {
    assert_eq!(number("2.0"), Ok(("", Json::Number(2.0f64))));
    assert_eq!(number("2.#"), Ok(("#", Json::Number(2.0f64))));
    assert_eq!(number("a2"), Err(NomErr::Error(JsonError::InvalidNumber)));
}

#[test]
pub fn test_string() {
    assert_eq!(string(r#""abc""#), Ok(("", r#"abc"#)));
    assert_eq!(string(r#""abc02""#), Ok(("", r#"abc02"#)));
    assert_eq!(string(r#""abc_def""#), Ok(("", r#"abc_def"#)));
    assert_eq!(string(r#""abc_\bef""#), Ok(("", r#"abc_\bef"#)));
    assert_eq!(string(r#""abc_\nef""#), Ok(("", r#"abc_\nef"#)));
    assert_eq!(string(r#""abc_\"ef""#), Ok(("", r#"abc_\"ef"#)));
    assert_eq!(string(r#""a""#), Ok(("", r#"a"#)));
    assert_eq!(string(r#""""#), Err(NomErr::Error(JsonError::InvalidString)));
    assert_eq!(string(r#"abc"#), Err(NomErr::Error(JsonError::InvalidString)));
}

#[test]
pub fn test_array() {
    assert_eq!(
        array(r#"["abc", 234, true, null]"#),
        Ok(("", Json::Array(vec![
            Json::String("abc".to_string()), 
            Json::Number(234f64),
            Json::Boolean(true),
            Json::Null
        ]))),
    );
    assert_eq!(
        array(r#"[ "abc" ]"#),
        Ok(("", Json::Array(vec![Json::String("abc".to_string())]))),
    );
    assert_eq!(
        array(r#"["abc", 234, true, null, "bug":"bug"]"#), 
        Err(NomErr::Error(JsonError::InvalidArray)),
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
    map.insert("arr".to_string(), Json::Array(vec![
        Json::String("abc".to_string()), 
        Json::Number(234f64),
        Json::Boolean(true),
        Json::Null
    ]));
    let mut sub_map = HashMap::new();
    sub_map.insert("str".to_string(), Json::String("value".to_string()));
    map.insert("object".to_string(), Json::Object(sub_map));
   
    assert_eq!(
        object(ok_data),
        Ok(("", Json::Object(map))),
    );
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

    let mut data = Json::Object(
        hashmap![
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
                            "driver".to_string() => Json::String("virtio-blk".to_string()), 
                            "source".to_string() => Json::String("focal-server-cloudimg-amd64.raw".to_string())
                        ]
                    ),
                    Json::Object(
                        hashmap![
                            "driver".to_string() => Json::String("virtio-net".to_string()), 
                            "mac".to_string() => Json::String("fa:16:3e:21:c0:c0".to_string())
                        ]
                    ),
                    Json::Object(
                        hashmap![
                            "driver".to_string() => Json::String("vfio".to_string()), 
                            "source".to_string() => Json::String("02:00.0".to_string())
                        ]
                    ),
                    Json::Object(
                        hashmap![
                            "driver".to_string() => Json::String("console".to_string()), 
                            "type".to_string() => Json::String("tty".to_string())
                        ]
                    ),
                ] 
            ),
            "os".to_string() => Json::Object(
                hashmap![
                        "kernel".to_string() => Json::String("/tmp/test-vm/vmlinux.bin".to_string()),
                        "initrd".to_string() => Json::Null,
                        "rootfs".to_string() => Json::String("/tmp/test-vm/bionic.rootfs.ext4".to_string()),
                        "cmdline".to_string() => Json::String("console=ttyS0 reboot=k panic=1 pci=off".to_string()) 
                ]
            ),
            "vmm".to_string() => Json::Object(hashmap![])
        ]
    );
    assert_eq!(object, Ok(data));
}