use std::{
    collections::HashMap,
    error::Error,
    fs::{create_dir_all, File},
    io::{self, Read, Write},
    path::Path,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::storage::Value;

#[derive(Debug)]
struct RdbHeader {
    version: u32,
}

#[derive(Debug)]
struct RdbEntry {
    key: String,
    value: String,
    expiry: Option<Instant>,
}

fn parse_rdb_header(buffer: &mut Bytes) -> Result<RdbHeader, String> {
    // "REDIS" + 4 bytes for version
    if buffer.remaining() < 9 {
        return Err("File too short".into());
    }
    let magic = buffer.split_to(5);
    if *magic != *b"REDIS" {
        return Err("Invalid RDB file: not starting with REDIS".into());
    }

    let version = buffer.get_u32();
    Ok(RdbHeader { version })
}

// TODO: add expiry
fn parse_rdb_entry(buffer: &mut Bytes) -> Result<Option<RdbEntry>, String> {
    if !buffer.has_remaining() {
        return Ok(None);
    }
    let data_type = buffer.get_u8();
    if data_type == 0xFF {
        return Ok(None); // EOF marker
    }
    match data_type {
        0x00 => parse_rdb_string(buffer, None),
        0xFD => {
            let expiry = parse_expiry(buffer, true)?;
            parse_rdb_string(buffer, expiry)
        }
        0xFC => {
            let expiry = parse_expiry(buffer, false)?;
            parse_rdb_string(buffer, expiry)
        }
        _ => Err("unknown data type".into()),
    }
}

fn parse_rbd_metadata(buffer: &mut Bytes) -> Result<(), String> {
    while buffer.remaining() > 0 {
        let byte = buffer.get_u8();
        if byte == 0xFE {
            return Ok(());
        }
    }
    Err("Metadata section did not end correctly".into())
}

fn parse_rbd_database_start(buffer: &mut Bytes) -> Result<(), String> {
    while buffer.remaining() > 0 {
        let byte = buffer.get_u8();
        if byte == 0xFB {
            // Skipping hash map size + expiry size
            buffer.get_u8();
            buffer.get_u8();
            return Ok(());
        }
    }
    Err("Database section did not start correctly".into())
}

fn parse_string(buffer: &mut Bytes) -> Result<String, String> {
    let str_len = buffer.get_u8();
    if buffer.remaining() < str_len as usize {
        return Err("File truncated while reading key".into());
    }
    let str_bytes = buffer.split_to(str_len as usize);
    Ok(String::from_utf8_lossy(str_bytes.as_ref()).to_string())
}

fn parse_rdb_string(
    buffer: &mut Bytes,
    expiry: Option<Instant>,
) -> Result<Option<RdbEntry>, String> {
    let key = parse_string(buffer)?;
    let value = parse_string(buffer)?;
    Ok(Some(RdbEntry { key, value, expiry }))
}

pub fn parse_rdb_file(_fn: &str) -> Result<HashMap<String, Value>, Box<dyn Error>> {
    let mut f = if let Ok(f) = File::open(_fn) {
        f
    } else {
        return Ok(HashMap::new());
    };

    let mut buf = Vec::new();

    f.read_to_end(&mut buf)?;
    if buf.is_empty() {
        return Ok(HashMap::new());
    }

    let mut buf = Bytes::from(buf);

    // Header
    let v = parse_rdb_header(&mut buf)?;
    println!("Version: {}", v.version);

    let mut m = HashMap::new();

    // Metadata
    parse_rbd_metadata(&mut buf)?;

    // Database
    parse_rbd_database_start(&mut buf)?;
    loop {
        match parse_rdb_entry(&mut buf) {
            Ok(Some(entry)) => {
                println!("Parsed Entry: {:?}", entry);
                m.insert(
                    entry.key,
                    Value {
                        value: entry.value,
                        expiry: entry.expiry,
                    },
                );
            }
            Ok(None) => {
                println!("End of RDB file");
                break;
            }
            Err(err) => {
                eprintln!("Error parsing entry: {}", err);
                break;
            }
        }
    }

    Ok(m)
}

pub fn write_rdb_file(_fn: &str, map: HashMap<String, Value>) -> Result<(), io::Error> {
    let path = Path::new(_fn);
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    let mut f = File::create(_fn)?;
    let mut buf = BytesMut::new();

    buf.extend_from_slice(b"REDIS\x00\x00\x00\x09");
    buf.extend_from_slice(b"\xFA\xFE\x01\x01");

    for (k, v) in map.iter() {
        buf.put_u8(0x00);
        write_rdb_string(&mut buf, k);
        write_rdb_string(&mut buf, &v.value);
    }

    buf.put_u8(0xFF);

    f.write_all(&buf)?;

    Ok(())
}

fn write_rdb_string(buf: &mut BytesMut, k: &str) {
    let key_len = k.len() as u8;
    buf.put_u8(key_len);
    buf.extend_from_slice(k.as_bytes());
}

fn parse_expiry(buf: &mut Bytes, seconds: bool) -> Result<Option<Instant>, String> {
    let system_time = if seconds {
        let expiry_bytes = buf.split_to(4);
        let expiry_timestamp = u32::from_le_bytes(expiry_bytes[..].try_into().unwrap()) as u64;
        UNIX_EPOCH + Duration::from_secs(expiry_timestamp)
    } else {
        let expiry_bytes = buf.split_to(8); // Extracts the first 8 bytes
        let expiry_timestamp = u64::from_le_bytes(expiry_bytes[..].try_into().unwrap());
        UNIX_EPOCH + Duration::from_millis(expiry_timestamp)
    };

    buf.get_u8();

    let now = SystemTime::now();
    match system_time.duration_since(now) {
        Ok(duration) => Ok(Some(Instant::now() + duration)),
        Err(_) => Ok(Some(Instant::now() - Duration::from_secs(1))),
    }
}

mod tests {
    // use super::*;

    #[test]
    fn should_read_header_string() {
        let tmp_file = "tmp.rdb";
        let given = b"REDIS\x00\x00\x00\x09\x00\x03key\x05value\xFF";
        let mut f = File::create(tmp_file).unwrap();
        f.write_all(given).unwrap();
        drop(f);
        let result = parse_rdb_file(tmp_file).unwrap();
        let result = result["key"].clone();
        assert_eq!(result.value, "value".to_string());
    }

    #[test]
    fn should_write_hash() {
        let tmp_file = "tmp.rdb";
        let mut given = HashMap::new();
        given.insert(
            "foo".to_string(),
            Value {
                value: "bar".to_string(),
                expiry: None,
            },
        );

        let result = write_rdb_file(tmp_file, given);
        assert!(result.is_ok());

        let mut f = File::open(tmp_file).unwrap();
        let mut s = Vec::new();
        f.read_to_end(&mut s); // Should match -> REDIS\x00\x00\x00\x09\x00\x03key\x05value\xFF
        drop(f);

        let expected = b"REDIS\x00\x00\x00\x09\xFArandom\xFE\x00\x03key\x05value\xFF";
        assert_eq!(s, expected);
    }

    #[test]
    fn should_read_header_version() {
        let given = b"REDIS\x00\x00\x00\x09";
        let mut given = Bytes::from(given.as_slice());
        let expected = RdbHeader { version: 9 };
        let result = parse_rdb_header(&mut given);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().version, expected.version);
    }

    #[test]
    fn should_read_header_entry() {
        let given = b"\x00\x03key\x05value\xFF";
        let mut given = Bytes::from(given.as_slice());
        let expected = RdbEntry {
            key: "key".to_string(),
            value: "value".to_string(),
        };
        let result = parse_rdb_entry(&mut given).unwrap().unwrap();

        assert_eq!(result.key, expected.key);
        assert_eq!(result.value, expected.value);
    }
}
