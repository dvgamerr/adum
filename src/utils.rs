use failure::{bail, format_err, Fallible};
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use url::{ParseError, Url};

pub fn parse_url(url: &str) -> Result<Url, ParseError> {
    match Url::parse(url) {
        Ok(url) => Ok(url),
        Err(error) if error == ParseError::RelativeUrlWithoutBase => {
            let url_with_base = format!("{}{}", "http://", url);
            Url::parse(url_with_base.as_str())
        }
        Err(error) => Err(error),
    }
}

pub fn gen_error(msg: String) -> Fallible<()> {
    bail!(msg)
}

pub fn get_file_handle(fname: &str, resume_download: bool, append: bool) -> io::Result<File> {
    if resume_download && Path::new(fname).exists() {
        if append {
            match OpenOptions::new().append(true).open(fname) {
                Ok(file) => Ok(file),
                Err(error) => Err(error),
            }
        } else {
            match OpenOptions::new().write(true).open(fname) {
                Ok(file) => Ok(file),
                Err(error) => Err(error),
            }
        }
    } else {
        match OpenOptions::new().write(true).create(true).open(fname) {
            Ok(file) => Ok(file),
            Err(error) => Err(error),
        }
    }
}

pub fn decode_percent_encoded_data(data: &str) -> Fallible<String> {
    let mut unescaped_bytes: Vec<u8> = Vec::new();
    let bytes = data.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let value = match bytes.get(index + 1..index + 3) {
                Some([high, low]) if high.is_ascii_hexdigit() && low.is_ascii_hexdigit() => {
                    (hex_value(*high) << 4) | hex_value(*low)
                }
                _ => bail!("invalid percent encoding at offset {} in {:?}", index, data),
            };
            unescaped_bytes.push(value);
            index += 3;
        } else {
            unescaped_bytes.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(unescaped_bytes)
        .map_err(|_| format_err!("percent-decoded data is not valid UTF-8: {:?}", data))
}

fn hex_value(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => byte - b'A' + 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_percent_encoded_data() {
        let x = "hello%20world";
        let y = decode_percent_encoded_data(x).unwrap();
        assert_eq!(&y, "hello world");
    }

    #[test]
    fn test_decode_percent_encoded_data_rejects_invalid_encoding() {
        for encoded in ["%", "%2", "%GG", "a%+fb", "%+1", "%-1", "%é", "%FF"] {
            assert!(decode_percent_encoded_data(encoded).is_err(), "{}", encoded);
        }
    }

    #[test]
    fn test_decode_percent_encoded_data_valid_edge_cases() {
        assert_eq!(decode_percent_encoded_data("a%41").unwrap(), "aA");
        assert_eq!(decode_percent_encoded_data("%e0%B8%81").unwrap(), "ก");
        assert_eq!(decode_percent_encoded_data("").unwrap(), "");
    }

    #[test]
    fn test_decode_percent_encoded_data_error_has_context() {
        let err = decode_percent_encoded_data("ab%GG")
            .unwrap_err()
            .to_string();
        assert!(err.contains("offset 2"), "{}", err);
    }
}
