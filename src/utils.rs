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
            let encoded_byte = bytes
                .get(index + 1..index + 3)
                .ok_or_else(|| format_err!("incomplete percent encoding"))?;
            let encoded_byte = std::str::from_utf8(encoded_byte)?;
            unescaped_bytes.push(u8::from_str_radix(encoded_byte, 16)?);
            index += 3;
        } else {
            unescaped_bytes.push(bytes[index]);
            index += 1;
        }
    }
    Ok(String::from_utf8(unescaped_bytes)?)
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
        for encoded in ["%", "%2", "%GG"] {
            assert!(decode_percent_encoded_data(encoded).is_err());
        }
    }
}
