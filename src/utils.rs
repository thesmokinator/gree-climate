pub fn pkcs7_pad(data: &[u8], block_size: usize) -> Vec<u8> {
    let pad_len = block_size - (data.len() % block_size);
    let pad_byte = pad_len as u8;
    let mut padded = Vec::with_capacity(data.len() + pad_len);
    padded.extend_from_slice(data);
    padded.extend(std::iter::repeat_n(pad_byte, pad_len));
    padded
}

pub fn pkcs7_unpad(data: &[u8]) -> Result<&[u8], crate::Error> {
    if data.is_empty() {
        return Err(crate::Error::Crypto("empty data for unpadding".into()));
    }
    let pad_len = *data.last().unwrap() as usize;
    if pad_len == 0 || pad_len > 16 || pad_len > data.len() {
        return Err(crate::Error::Crypto(format!(
            "invalid PKCS7 padding length: {pad_len}"
        )));
    }
    for &byte in &data[data.len() - pad_len..] {
        if byte as usize != pad_len {
            return Err(crate::Error::Crypto(
                "invalid PKCS7 padding bytes".into(),
            ));
        }
    }
    Ok(&data[..data.len() - pad_len])
}

#[allow(dead_code)]
pub(crate) const TEMP_MIN: u8 = 8;
#[allow(dead_code)]
pub(crate) const TEMP_MAX: u8 = 30;
#[allow(dead_code)]
pub(crate) const TEMP_OFFSET: i16 = 40;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub(crate) struct TempRecord {
    pub fahrenheit: i32,
    pub set_point: i32,
    pub bit: i32,
}

#[allow(dead_code)]
const TEMP_MIN_TABLE_F: i32 = -76;
#[allow(dead_code)]
const TEMP_MAX_TABLE_F: i32 = 140;

#[allow(dead_code)]
pub(crate) fn generate_temperature_record(temp_f: i32) -> TempRecord {
    let temp_c = (temp_f - 32) * 5 / 9;
    TempRecord {
        fahrenheit: temp_f,
        set_point: temp_c,
        bit: i32::from(temp_c - (temp_f - 32) * 5 / 9 > 0),
    }
}

#[allow(dead_code)]
pub(crate) fn build_temperature_table() -> Vec<TempRecord> {
    (TEMP_MIN_TABLE_F..=TEMP_MAX_TABLE_F)
        .map(generate_temperature_record)
        .collect()
}

pub(crate) fn reset_trailing_garbage(s: &str) -> String {
    if let Some(pos) = s.rfind('}') {
        s[..=pos].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkcs7_pad_empty() {
        let result = pkcs7_pad(b"", 16);
        assert_eq!(result.len(), 16);
        assert!(result.iter().all(|&b| b == 16));
    }

    #[test]
    fn test_pkcs7_pad_partial() {
        let data = b"hello";
        let result = pkcs7_pad(data, 16);
        assert_eq!(result.len(), 16);
        let pad_len = (16 - data.len()) as u8;
        assert_eq!(result[..5], *b"hello");
        for &byte in &result[5..] {
            assert_eq!(byte, pad_len);
        }
    }

    #[test]
    fn test_pkcs7_pad_full_block() {
        let data = b"1234567890ABCDEF"; // exactly 16 bytes
        let result = pkcs7_pad(data, 16);
        assert_eq!(result.len(), 32);
        assert_eq!(&result[..16], data);
        for &byte in &result[16..] {
            assert_eq!(byte, 16);
        }
    }

    #[test]
    fn test_pkcs7_unpad_valid() {
        let mut padded = vec![1, 2, 3];
        padded.extend(std::iter::repeat_n(13u8, 13));
        let result = pkcs7_unpad(&padded).unwrap();
        assert_eq!(result, &[1, 2, 3]);
    }

    #[test]
    fn test_pkcs7_unpad_invalid() {
        let data = [1, 2, 3, 99];
        assert!(pkcs7_unpad(&data).is_err());
    }

    #[test]
    fn test_reset_trailing_garbage() {
        let result = reset_trailing_garbage(r#"{"key": "value"}xxxx"#);
        assert_eq!(result, r#"{"key": "value"}"#);
    }

    #[test]
    fn test_generate_temperature_record() {
        let rec = generate_temperature_record(68); // 20C
        assert_eq!(rec.set_point, 20);
    }

    #[test]
    fn test_temperature_table_coverage() {
        let table = build_temperature_table();
        assert_eq!(table.len(), (TEMP_MAX_TABLE_F - TEMP_MIN_TABLE_F + 1) as usize);
    }
}
