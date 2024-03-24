use std::collections::HashMap;
use std::fmt;
use std::string::FromUtf8Error;

#[derive(Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number {
        integer: i64,
        fraction: i64,
        precision: usize,
        exponent: i64,
    },
    String(String),
    Array(Vec<Json>),
    Object(Dict),
}

impl fmt::Display for Json {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Json::Null => write!(fmt, "null"),
            Json::Bool(b) => write!(fmt, "{}", b),
            Json::Number {
                integer,
                fraction,
                precision,
                exponent,
            } => print_number(fmt, *integer, *fraction, *precision, *exponent),
            Json::String(s) => write!(fmt, "\"{}\"", escape(s)),
            Json::Array(arr) => print_json_array(arr, fmt),
            Json::Object(dict) => print_json_object(dict, fmt),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyString,
    CharMismatch { expected: u8, actual: u8 },
    HexCharExpected,
    UtfDecodingError(FromUtf8Error),
    NullExpected,
    TrueExpected,
    FalseExpected,
    ExponentRequired,
    UnrecognisedEscapeSequence(u8),
    InvalidValue,
    OutOfBounds,
    Garbage,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::EmptyString => write!(fmt, "Empty string parsed"),
            Error::CharMismatch { expected, actual } => {
                write!(
                    fmt,
                    "Expected {}, but read {}",
                    char::from(*expected),
                    char::from(*actual)
                )
            }
            Error::HexCharExpected => write!(fmt, "Expected hex char"),
            Error::UtfDecodingError(error) => write!(fmt, "{}", error),
            Error::NullExpected => write!(fmt, "Expected null"),
            Error::TrueExpected => write!(fmt, "Expected true"),
            Error::FalseExpected => write!(fmt, "Expected false"),
            Error::ExponentRequired => write!(fmt, "Exponent required"),
            Error::UnrecognisedEscapeSequence(ch) => {
                write!(fmt, "Unrecognised escape sequence \\{}", ch)
            }
            Error::InvalidValue => write!(fmt, "Invalid value"),
            Error::OutOfBounds => write!(fmt, "Out of bounds read attempt"),
            Error::Garbage => write!(fmt, "garbage found at the end of string"),
        }
    }
}

type Idx = usize;

pub type Dict = HashMap<String, Json>;

type JsonPart<'a> = Result<(Json, &'a [u8]), (Error, &'a [u8])>;

type JsonResult = Result<Json, (Error, Idx)>;

fn is_alpha(ch: u8) -> bool {
    (b'a'..=b'z').contains(&ch) || (b'A'..=b'Z').contains(&ch)
}

fn is_digit(ch: u8) -> bool {
    (b'0'..=b'9').contains(&ch)
}

fn is_hex(ch: u8) -> bool {
    (b'0'..=b'9').contains(&ch) || (b'a'..=b'f').contains(&ch) || (b'A'..=b'F').contains(&ch)
}

fn is_ws(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r'
}

fn take(s: &[u8], f: impl Fn(u8) -> bool) -> Result<(&[u8], &[u8]), (Error, &[u8])> {
    let mut i = 0;
    while i < s.len() && f(s[i]) {
        i += 1;
    }
    if i > 0 {
        Ok(s.split_at(i))
    } else {
        Err((Error::EmptyString, s))
    }
}

fn ask(
    s: &[u8],
    f: impl Fn(&[u8]) -> Result<(Option<u8>, &[u8]), (Error, &[u8])>,
) -> Result<(String, &[u8]), (Error, &[u8])> {
    let mut data = Vec::<u8>::new();
    let mut cont = s;
    loop {
        if cont.is_empty() {
            break;
        }
        if let (Some(c), s) = f(cont)? {
            data.push(c);
            cont = s;
        } else {
            break;
        }
    }
    match String::from_utf8(data) {
        Ok(string) => Ok((string, cont)),
        Err(error) => Err((Error::UtfDecodingError(error), cont)),
    }
}

fn skip(s: &[u8], f: impl Fn(u8) -> bool) -> &[u8] {
    let mut s = s;
    while let Some((c, cont)) = s.split_first() {
        if f(*c) {
            s = cont
        } else {
            break;
        }
    }
    s
}

fn skip_ws(s: &[u8]) -> &[u8] {
    skip(s, is_ws)
}

fn chr(s: &[u8], ch: u8) -> Result<(u8, &[u8]), (Error, &[u8])> {
    match s.split_first() {
        Some((c, cont)) => {
            if *c == ch {
                Ok((ch, cont))
            } else {
                Err((
                    Error::CharMismatch {
                        expected: ch,
                        actual: *c,
                    },
                    s,
                ))
            }
        }
        None => Err((Error::OutOfBounds, s)),
    }
}

fn parse_null(s: &[u8]) -> JsonPart {
    match take(s, is_alpha) {
        Ok((b"null", s)) => Ok((Json::Null, s)),
        _ => Err((Error::NullExpected, s)),
    }
}

fn parse_true(s: &[u8]) -> JsonPart {
    match take(s, is_alpha) {
        Ok((b"true", s)) => Ok((Json::Bool(true), s)),
        _ => Err((Error::TrueExpected, s)),
    }
}

fn parse_false(s: &[u8]) -> JsonPart {
    match take(s, is_alpha) {
        Ok((b"false", s)) => Ok((Json::Bool(false), s)),
        _ => Err((Error::FalseExpected, s)),
    }
}

fn parse_fraction(s: &[u8]) -> Result<(i64, usize, &[u8]), (Error, &[u8])> {
    match take(s, is_digit) {
        Ok((fractions, s)) => Ok((to_i64(fractions), fractions.len(), s)),
        _ => Ok((0, 0, s)),
    }
}

fn parse_exponent(s: &[u8]) -> Result<(i64, &[u8]), (Error, &[u8])> {
    match take(s, is_digit) {
        Ok((exponent, s)) => Ok((to_i64(exponent), s)),
        _ => Err((Error::ExponentRequired, s)),
    }
}

fn to_i64(s: &[u8]) -> i64 {
    std::str::from_utf8(s).unwrap().parse::<i64>().unwrap()
}

fn parse_number_parts(s: &[u8]) -> Result<(i64, i64, usize, i64, &[u8]), (Error, &[u8])> {
    let (ints, s) = take(s, is_digit)?;
    let ints = to_i64(ints);
    let (fractions, precision, s) = match chr(s, b'.') {
        Ok((_, s)) => parse_fraction(s)?,
        _ => (0, 0, s),
    };
    let (exponent, s) = match chr(s, b'e') {
        Ok((_, s)) => parse_exponent(s)?,
        _ => (0, s),
    };
    Ok((ints, fractions, precision, exponent, s))
}

fn parse_number(s: &[u8]) -> JsonPart {
    let (ints, fractions, precision, exponent, s) = parse_number_parts(s)?;
    let json = Json::Number {
        integer: ints,
        fraction: fractions,
        precision: precision,
        exponent: exponent,
    };
    Ok((json, s))
}

fn parse_negative_number(s: &[u8]) -> JsonPart {
    let (_, s) = chr(s, b'-')?;
    let (ints, fractions, precision, exponent, s) = parse_number_parts(s)?;
    let json = Json::Number {
        integer: -ints,
        fraction: fractions,
        precision: precision,
        exponent: exponent,
    };
    Ok((json, s))
}

fn hex(s: &[u8]) -> Result<(u8, &[u8]), (Error, &[u8])> {
    match s.split_first() {
        None => Err((Error::OutOfBounds, s)),
        Some((c, cont)) => {
            if is_hex(*c) {
                Ok((*c, cont))
            } else {
                Err((Error::HexCharExpected, s))
            }
        }
    }
}

fn hexword(s: &[u8]) -> Result<(u8, &[u8]), (Error, &[u8])> {
    let (_, s) = hex(s)?;
    let (_, s) = hex(s)?;
    let (_, s) = hex(s)?;
    let (_, s) = hex(s)?;
    Ok((b'?', s)) // We don't support unicode, just return question mark
}

fn string_char(s: &[u8]) -> Result<(Option<u8>, &[u8]), (Error, &[u8])> {
    match s.split_first() {
        None => Err((Error::OutOfBounds, s)),
        Some((b'"', _)) => Ok((None, &s)),
        Some((b'\\', s)) => {
            match s.split_first() {
                None => Err((Error::OutOfBounds, s)),
                Some((b'"', s)) => Ok((Some(b'"'), s)),
                Some((b'\\', s)) => Ok((Some(b'\\'), s)),
                Some((b'/', s)) => Ok((Some(b'/'), s)),
                Some((b'b', s)) => Ok((Some(0x08), s)), // rust doesn't support \b
                Some((b'f', s)) => Ok((Some(0x0C), s)), // rust doesn't support \f
                Some((b'n', s)) => Ok((Some(b'\n'), s)),
                Some((b'r', s)) => Ok((Some(b'\r'), s)),
                Some((b't', s)) => Ok((Some(b'\t'), s)),
                Some((b'u', s)) => {
                    let (c, s) = hexword(s)?;
                    Ok((Some(c), s))
                }
                _ => Err((Error::UnrecognisedEscapeSequence(s[1]), s)),
            }
        }
        Some((c, s)) => Ok((Some(*c), s)),
    }
}

fn parse_string_raw(s: &[u8]) -> Result<(String, &[u8]), (Error, &[u8])> {
    let (_, s) = chr(s, b'"')?;
    let (contents, s) = ask(s, string_char)?;
    let (_, s) = chr(s, b'"')?;
    Ok((contents, s))
}

fn parse_string(s: &[u8]) -> JsonPart {
    let (contents, s) = parse_string_raw(s)?;
    Ok((Json::String(String::from(contents)), s))
}

fn parse_array_items(s: &[u8], value: Json) -> Result<(Vec<Json>, &[u8]), (Error, &[u8])> {
    let mut items = vec![value];
    let mut cont = s;
    while let Ok((_, s)) = chr(skip_ws(cont), b',') {
        let (value, s) = parse_value(s)?;
        items.push(value);
        cont = s;
    }
    Ok((items, cont))
}

fn parse_array(s: &[u8]) -> JsonPart {
    let (_, s) = chr(s, b'[')?;
    let (items, s) = match parse_value(s) {
        Err((_, s)) => Ok((vec![], s)),
        Ok((value, s)) => parse_array_items(s, value),
    }?;
    let s = skip_ws(s);
    let (_, s) = chr(s, b']')?;
    Ok((Json::Array(items), s))
}

fn parse_key_value_pair(s: &[u8]) -> Result<(String, Json, &[u8]), (Error, &[u8])> {
    let s = skip_ws(s);
    let (key, s) = parse_string_raw(s)?;
    let s = skip_ws(s);
    let (_, s) = chr(s, b':')?;
    let (value, s) = parse_value(s)?;
    Ok((key, value, s))
}

fn parse_object_items(s: &[u8], key: String, value: Json) -> Result<(Dict, &[u8]), (Error, &[u8])> {
    let mut items = Dict::new();
    items.insert(key, value);
    let mut cont = s;
    while let Ok((_, s)) = chr(skip_ws(cont), b',') {
        let (key, value, s) = parse_key_value_pair(s)?;
        items.insert(key, value);
        cont = s;
    }
    Ok((items, cont))
}

fn parse_object(s: &[u8]) -> JsonPart {
    let (_, s) = chr(s, b'{')?;
    let s = skip_ws(s);
    let (pairs, s) = if let Some(b'"') = s.first() {
        let (key, value, s) = parse_key_value_pair(s)?;
        parse_object_items(s, key, value)?
    } else {
        (Dict::new(), s)
    };
    let s = skip_ws(s);
    let (_, s) = chr(s, b'}')?;
    Ok((Json::Object(pairs), s))
}

fn parse_value(s: &[u8]) -> JsonPart {
    let s = skip_ws(s);
    match s.first() {
        Some(b'n') => parse_null(s),
        Some(b't') => parse_true(s),
        Some(b'f') => parse_false(s),
        Some(b'0'..=b'9') => parse_number(s),
        Some(b'-') => parse_negative_number(s),
        Some(b'"') => parse_string(s),
        Some(b'[') => parse_array(s),
        Some(b'{') => parse_object(s),
        None => Err((Error::OutOfBounds, s)),
        _ => Err((Error::InvalidValue, s)),
    }
}

pub fn parse_bytes(s: &[u8]) -> JsonResult {
    let len = s.len();
    let s = skip_ws(s);
    if s.is_empty() {
        return Err((Error::EmptyString, 0));
    }
    match parse_value(s) {
        Ok((json, s)) => {
            let s = skip_ws(s);
            if s.is_empty() {
                Ok(json)
            } else {
                Err((Error::Garbage, (len - s.len())))
            }
        }
        Err((error, s)) => Err((error, (len - s.len()))),
    }
}

pub fn parse_str(s: &str) -> JsonResult {
    parse_bytes(s.as_bytes())
}

fn escape(s: &String) -> String {
    let esc = |c: char| match c {
        '\"' => ("\\\"").chars().collect(),
        '\\' => ("\\\\").chars().collect(),
        '\r' => ("\\r").chars().collect(),
        '\n' => ("\\n").chars().collect(),
        '\t' => ("\\t").chars().collect(),
        '\u{0008}' => ("\\b").chars().collect(),
        '\u{000C}' => ("\\f").chars().collect(),
        c => vec![c],
    };
    s.chars().flat_map(esc).collect()
}

fn print_number(
    fmt: &mut fmt::Formatter,
    integer: i64,
    fraction: i64,
    precision: usize,
    exponent: i64,
) -> fmt::Result {
    let r = write!(fmt, "{}", integer);
    if precision > 0 {
        write!(fmt, ".{}", fraction)?;
    }
    if exponent != 0 {
        write!(fmt, "e{}", exponent)?;
    }
    r
}

fn print_json_array(arr: &Vec<Json>, fmt: &mut fmt::Formatter) -> fmt::Result {
    write!(fmt, "[")?;
    if arr.len() > 0 {
        write!(fmt, "{}", arr[0])?;
        if arr.len() > 1 {
            for item in arr.iter() {
                write!(fmt, ", {}", item)?;
            }
        }
    }
    write!(fmt, "]")
}

fn print_json_object(obj: &Dict, fmt: &mut fmt::Formatter) -> fmt::Result {
    write!(fmt, "{{")?;
    let mut iter = obj.iter();
    if let Some((key, value)) = iter.next() {
        write!(fmt, "\"{}\": {}", escape(key), value)?;
        for (key, value) in iter {
            write!(fmt, ", \"{}\": {}", escape(key), value)?;
        }
    }
    write!(fmt, "}}")
}
