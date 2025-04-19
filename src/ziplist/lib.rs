use crate::ziplist::{*};
use crate::ziplist::error::ZipListError;

pub enum Content {
    String(String),
    Integer(i64),
}

pub fn decode_prev_len_size(ptr: &[u8]) -> u32 {
    if ptr[0] < ZIP_BIG_PREVLEN {
        1
    } else {
        5
    }
}

pub fn decode_prev_len(ptr: &[u8]) -> (u32, u32) {
    let prev_len_size = decode_prev_len_size(ptr);
    let prev_len = if prev_len_size == 1 {
        ptr[0] as u32
    } else {
        u32::from_le_bytes([ptr[1], ptr[2], ptr[3], ptr[4]])
    };
    (prev_len_size, prev_len)
}

pub fn entry_encoding(ptr: &[u8]) -> u8 {
    let mut encoding = ptr[0];
    if encoding < ZIP_STR_MASK {
        encoding &= ZIP_STR_MASK;
    }
    encoding
}


/* Decode the entry encoding type and data length (string length for strings,
 * number of bytes used for the integer for integer entries) encoded in 'ptr'.
 * The 'encoding' variable is input, extracted by the caller, the 'lensize'
 * variable will hold the number of bytes required to encode the entry
 * length, and the 'len' variable will hold the entry length.
 * On invalid encoding error, lensize is set to 0. */
pub fn decode_length(ptr: &[u8], encoding: u8) -> (u32, u32) {
    if encoding < ZIP_STR_MASK {
        match encoding {
            ZIP_STR_06B => {
                (1, (ptr[0] & 0x3f) as u32)
            }
            ZIP_STR_14B => {
                let len = (((ptr[0] & 0x3f) as u32) << 8) | (ptr[1] as u32);
                (2, len)
            }
            ZIP_STR_32B => {
                let len = u32::from_le_bytes([ptr[1], ptr[2], ptr[3], ptr[4]]);
                (5, len)
            }
            _ => (0, 0), // bad encoding
        }
    } else {
        let len_size = 1;
        let len = match encoding {
            ZIP_INT_8B => 1,
            ZIP_INT_16B => 2,
            ZIP_INT_24B => 3,
            ZIP_INT_32B => 4,
            ZIP_INT_64B => 8,
            imm if imm >= ZIP_INT_IMM_MIN && imm <= ZIP_INT_IMM_MAX => 0,
            _ => return (0, 0), // bad encoding
        };
        (len_size, len)
    }
}

#[inline]
pub fn encoding_len_size(encoding: u8) -> u32 {
    match encoding {
        ZIP_INT_8B | ZIP_INT_16B | ZIP_INT_24B | ZIP_INT_32B | ZIP_INT_64B => 1,
        ZIP_INT_IMM_MIN..=ZIP_INT_IMM_MAX => 1,
        ZIP_STR_06B => 1,
        ZIP_STR_14B => 2,
        ZIP_STR_32B => 5,
        _ => ZIP_ENCODING_SIZE_INVALID as u32,
    }
}

fn string_to_number(s: &str) -> Result<i64, ZipListError> {
    let b = s.as_bytes();
    let s_len = b.len();
    let mut negative = false;
    let mut v: u64 = 0;

    if s_len == 0 || s_len >= LONG_STR_SIZE {
        return Err(ZipListError::InValidString);
    }
    if s_len == 1 && b[0] == b'0' {
        return Ok(0);
    }

    let mut p_len = 0;
    if b[p_len] == b'-' {
        negative = true;
        p_len += 1;
    }
    if b[p_len] >= b'1' && b[p_len] <= b'9' {
        v = (b[p_len] - b'0') as u64;
        p_len += 1;
    } else {
        return Err(ZipListError::InvalidFirstDigit);
    }

    while p_len < s_len {
        let ch = b[p_len];
        if ch < b'0' || ch > b'9' {
            return Err(ZipListError::InvalidChar);
        }
        let digit = (ch - b'0') as u64;
        if v > u64::MAX / 10 {
            return Err(ZipListError::OverFlowMul);
        }
        v *= 10;
        if v > u64::MAX - digit {
            return Err(ZipListError::OverFlowAdd)
        }
        v += digit;
        p_len += 1;
    }

    if negative {
        if v > ((-(i64::MIN + 1) as u64) + 1) {
            return Err(ZipListError::OverFlowNegative);
        }
        Ok(-(v as i64))
    } else {
        if v > i64::MAX as u64 {
            return Err(ZipListError::OverFlowPositive);
        }
        Ok(v as i64)
    }
}

/// Return the integer value and its encoding
pub fn try_encoding(entry: &str) -> Option<(i64, u8)> {
    let len = entry.len();
    if len == 0 || len >= 32 {
        return None;
    }
    match string_to_number(entry) {
        Ok(value) => {
            let encoding = if value >= 0 && value < 12 {
                ZIP_INT_IMM_MIN + value as u8
            } else if value >= i8::MIN as i64 && value <= i8::MAX as i64 {
                ZIP_INT_8B
            } else if value >= i16:: MIN as i64 && value <= i16::MAX as i64 {
                ZIP_INT_16B
            } else if value >= INT_24_MIN && value <= INT_24_MAX {
                ZIP_INT_24B
            } else if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
                ZIP_INT_32B
            } else {
                ZIP_INT_64B
            };
            Some((value, encoding))
        }
        Err(_) => {
            None
        }
    }
}

#[inline]
pub fn int_size(encoding: u8) -> u32 {
    match encoding {
        ZIP_INT_8B => 1,
        ZIP_INT_16B => 2,
        ZIP_INT_24B => 3,
        ZIP_INT_32B => 4,
        ZIP_INT_64B => 8,
        _ => 0,
    };
    if encoding >= ZIP_INT_IMM_MIN && encoding <= ZIP_INT_IMM_MAX {
        return 0;
    }
    0
    //panic!("unreachable code reached");
}

pub fn store_prev_entry_length_large(data: Option<&mut [u8]>, len: u32) -> u32 {
    if let Some(p) = data {
        p.to_vec()[0] = ZIP_BIG_PREVLEN;
        p.to_vec()[1..5].copy_from_slice(&len.to_le_bytes());
    }
    (1 + size_of::<u32>()) as u32
}

pub fn store_prev_entry_length(data: Option<&mut [u8]>, len: u32) -> u32 {
    if let Some(p) = data {
        if len < ZIP_BIG_PREVLEN as u32 {
            p.to_vec()[0] = len as u8;
            1
        } else {
            store_prev_entry_length_large(Some(p), len)
        }
    } else {
        if len < ZIP_BIG_PREVLEN as u32 {
            1
        } else {
            (1 + size_of::<u32>()) as u32
        }
    }
}

pub fn is_string(encoding: u8) -> bool {
    encoding & ZIP_STR_MASK < ZIP_STR_MASK
}

pub fn store_entry_encoding(data: Option<&mut [u8]>, encoding: u8, raw_len: u32) -> u32 {
    let mut len = 1;
    let mut buf = vec![0u8; 5];

    if is_string(encoding) {
        if raw_len <= 0x3f {
            if data.is_none() {
                return len;
            }
            buf[0] = ZIP_STR_06B | (raw_len as u8);
        } else if raw_len <= 0x3fff {
            len += 1;
            if data.is_none() {
                return len;
            }
            buf[0] = ZIP_STR_14B | ((raw_len >> 8) as u8) & 0x3f;
            buf[1] = (raw_len & 0xff) as u8;
        } else {
            len += 4;
            if data.is_none() {
                return len;
            }
            buf[0] = ZIP_STR_32B;
            buf[1] = (raw_len >> 24) as u8;
            buf[2] = (raw_len >> 16) as u8;
            buf[3] = (raw_len >> 8) as u8;
            buf[4] = (raw_len & 0xff) as u8;
        }
    } else {
        if data.is_none() {
            return len;
        }
        buf[0] = encoding;
    }
    if let Some(data) = data {
        data[0..len as usize].copy_from_slice(&buf[..len as usize]);
    }
    len
}

pub fn prev_len_bytes_diff(ptr: &[u8], len: u32) -> i32 {
    let prev_len_size = decode_prev_len_size(ptr);
    store_prev_entry_length(None, len) as i32 - prev_len_size as i32
}

pub fn save_integer(ptr: &mut [u8], value: i64, encoding: u8) {
    match encoding {
        ZIP_INT_8B => {
            ptr[0] = value as i8 as u8;
        }
        ZIP_INT_16B => {
            let i16 = (value as i16).to_le_bytes();
            ptr[..2].copy_from_slice(&i16);
        }
        ZIP_INT_24B => {
            let i32 = ((value as u64) << 8).to_le_bytes();
            ptr[..3].copy_from_slice(&i32[1..]);
        }
        ZIP_STR_32B => {
            let i32 = (value as i32).to_le_bytes();
            ptr[..4].copy_from_slice(&i32);
        }
        ZIP_INT_64B => {
            let i64 = value.to_le_bytes();
            ptr[..8].copy_from_slice(&i64);
        }
        IMM if IMM >= ZIP_INT_IMM_MIN && IMM <= ZIP_INT_IMM_MAX => { }
        _ => {
            panic!("Invalid zip integer encoding");
        }
    }
}

pub fn incr_length(ptr: &mut [u8], incr: usize) {
    let len = u16::from_le_bytes(ptr[ZIPLIST_LENGTH_OFFSET..ZIPLIST_LENGTH_OFFSET + 2].try_into().unwrap());
    if len < u16::MAX {
        ptr[ZIPLIST_LENGTH_OFFSET..ZIPLIST_LENGTH_OFFSET + 2].copy_from_slice(&(len as usize + incr).to_le_bytes())
    }
}

#[cfg(test)]
mod test {
    use crate::ziplist::lib::string_to_number;

    #[test]
    fn to_number() {
        let s = "-1234567899999999".to_string();
        let n = string_to_number(&s);
        match n {
            Ok(n) => {
                println!("number: {}", n);
            }
            Err(e) => {
                println!("Err: {:?}", e);
            }
        }
    }
}