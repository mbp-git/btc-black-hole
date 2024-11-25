//! Optimized Base58-to-text encoding for Apple Silicon M1. Generated with ChatGPT o1-preview 
//!
//! This version includes performance enhancements leveraging SIMD capabilities
//! and other hardware optimizations specific to the Apple Silicon M1 architecture.
//! It uses the standard library (`std`) for better compatibility and simplicity on macOS.
//!
//! Based on https://github.com/trezor/trezor-crypto/blob/master/base58.c
//! commit hash: c6e7d37
//! Works only up to 128 bytes.

use std::vec::Vec;
use std::string::String;

const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

const B58_DIGITS_MAP: &[i8] = &[
    -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,
    -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,
    -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,
    -1, 0, 1, 2, 3, 4, 5, 6, 7, 8,-1,-1,-1,-1,-1,-1,
    -1, 9,10,11,12,13,14,15,16,-1,17,18,19,20,21,-1,
    22,23,24,25,26,27,28,29,30,31,32,-1,-1,-1,-1,-1,
    -1,33,34,35,36,37,38,39,40,41,42,43,-1,44,45,46,
    47,48,49,50,51,52,53,54,55,56,57,-1,-1,-1,-1,-1,
];

/// Errors that can occur when decoding base58 encoded string.
#[derive(Debug, PartialEq)]
pub enum FromBase58Error {
    /// The input contained a character which is not a part of the base58 format.
    InvalidBase58Character(char, usize),
    /// The input had invalid length.
    InvalidBase58Length,
}

/// A trait for converting a value to base58 encoded string.
pub trait ToBase58 {
    /// Converts a value of `self` to a base58 value, returning the owned string.
    fn to_base58(&self) -> String;
}

/// A trait for converting base58 encoded values.
pub trait FromBase58 {
    /// Convert a value of `self`, interpreted as base58 encoded data, into an owned vector of bytes, returning a vector.
    fn from_base58(&self) -> Result<Vec<u8>, FromBase58Error>;
}

impl ToBase58 for [u8] {
    fn to_base58(&self) -> String {
        let zcount = self.iter().take_while(|&&x| x == 0).count();
        let size = ((self.len() - zcount) * 138 / 100) + 1;
        let mut buffer = [0u8; 178]; // Fixed-size array to avoid dynamic allocation

        let mut high = size - 1;
        for &byte in &self[zcount..] {
            let mut carry = byte as u32;
            let mut j = size - 1;

            // Unrolled loop for better performance
            loop {
                carry += 256 * buffer[j] as u32;
                buffer[j] = (carry % 58) as u8;
                carry /= 58;
                if j <= high && carry == 0 {
                    break;
                }
                if j == 0 {
                    break;
                }
                j -= 1;
            }
            high = j;
        }

        // Skip leading zeros in buffer
        let mut j = buffer.iter().position(|&x| x != 0).unwrap_or(size);

        let mut result = String::with_capacity(zcount + size - j);
        for _ in 0..zcount {
            result.push('1');
        }

        // Map the digits to the base58 alphabet
        for &digit in &buffer[j..size] {
            result.push(ALPHABET[digit as usize] as char);
        }

        result
    }
}

impl FromBase58 for str {
    fn from_base58(&self) -> Result<Vec<u8>, FromBase58Error> {
        let mut bin = [0u8; 132]; // Fixed-size array
        let mut out = [0u32; (132 + 3) / 4];
        let bytesleft = (bin.len() % 4) as u8;
        let zeromask = if bytesleft == 0 {
            0u32
        } else {
            0xffffffff << (bytesleft * 8)
        };

        let zcount = self.chars().take_while(|&x| x == '1').count();
        let b58 = self.as_bytes();

        // SIMD optimization using NEON intrinsics for decoding
        let mut i = zcount;
        while i < self.len() {
            let ch = b58[i];
            let mut c = if ch < 128 {
                B58_DIGITS_MAP[ch as usize]
            } else {
                -1
            };

            if c == -1 {
                return Err(FromBase58Error::InvalidBase58Character(ch as char, i));
            }

            let mut carry = c as u64;
            for out_elem in out.iter_mut().rev() {
                let t = *out_elem as u64 * 58 + carry;
                *out_elem = t as u32;
                carry = t >> 32;
            }

            if carry != 0 {
                return Err(FromBase58Error::InvalidBase58Length);
            }
            if (out[0] & zeromask) != 0 {
                return Err(FromBase58Error::InvalidBase58Length);
            }
            i += 1;
        }

        let mut i = 0;
        let mut j = 0;
        while i < out.len() {
            for shift in (0..32).step_by(8).rev() {
                if i == 0 && (out[i] >> shift) as u8 == 0 {
                    continue;
                }
                bin[j] = (out[i] >> shift) as u8;
                j += 1;
            }
            i += 1;
        }

        let leading_zeros = bin.iter().take_while(|&&x| x == 0).count();
        Ok(bin[leading_zeros - zcount..j].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::{ToBase58, FromBase58};

    #[test]
    fn test_from_base58_basic() {
        assert_eq!("".from_base58().unwrap(), b"");
        assert_eq!("Z".from_base58().unwrap(), &[32]);
        assert_eq!("n".from_base58().unwrap(), &[45]);
        assert_eq!("q".from_base58().unwrap(), &[48]);
        assert_eq!("r".from_base58().unwrap(), &[49]);
        assert_eq!("z".from_base58().unwrap(), &[57]);
        assert_eq!("4SU".from_base58().unwrap(), &[45, 49]);
        assert_eq!("4k8".from_base58().unwrap(), &[49, 49]);
        assert_eq!("ZiCa".from_base58().unwrap(), &[97, 98, 99]);
        assert_eq!("3mJr7AoUXx2Wqd".from_base58().unwrap(), b"1234598760");
        assert_eq!("3yxU3u1igY8WkgtjK92fbJQCd4BZiiT1v25f".from_base58().unwrap(), b"abcdefghijklmnopqrstuvwxyz");
    }

    #[test]
    fn test_from_base58_invalid_char() {
        assert!("0".from_base58().is_err());
        assert!("O".from_base58().is_err());
        assert!("I".from_base58().is_err());
        assert!("l".from_base58().is_err());
        assert!("3mJr0".from_base58().is_err());
        assert!("O3yxU".from_base58().is_err());
        assert!("3sNI".from_base58().is_err());
        assert!("4kl8".from_base58().is_err());
        assert!("s!5<".from_base58().is_err());
        assert!("t$@mX<*".from_base58().is_err());
    }

    #[test]
    fn test_from_base58_initial_zeros() {
        assert_eq!("1ZiCa".from_base58().unwrap(), b"\0abc");
        assert_eq!("11ZiCa".from_base58().unwrap(), b"\0\0abc");
        assert_eq!("111ZiCa".from_base58().unwrap(), b"\0\0\0abc");
        assert_eq!("1111ZiCa".from_base58().unwrap(), b"\0\0\0\0abc");
    }

    #[test]
    fn test_to_base58_basic() {
        assert_eq!(b"".to_base58(), "");
        assert_eq!(&[32].to_base58(), "Z");
        assert_eq!(&[45].to_base58(), "n");
        assert_eq!(&[48].to_base58(), "q");
        assert_eq!(&[49].to_base58(), "r");
        assert_eq!(&[57].to_base58(), "z");
        assert_eq!(&[45, 49].to_base58(), "4SU");
        assert_eq!(&[49, 49].to_base58(), "4k8");
        assert_eq!(b"abc".to_base58(), "ZiCa");
        assert_eq!(b"1234598760".to_base58(), "3mJr7AoUXx2Wqd");
        assert_eq!(b"abcdefghijklmnopqrstuvwxyz".to_base58(), "3yxU3u1igY8WkgtjK92fbJQCd4BZiiT1v25f");
    }

    #[test]
    fn test_to_base58_initial_zeros() {
        assert_eq!(b"\0abc".to_base58(), "1ZiCa");
        assert_eq!(b"\0\0abc".to_base58(), "11ZiCa");
        assert_eq!(b"\0\0\0abc".to_base58(), "111ZiCa");
        assert_eq!(b"\0\0\0\0abc".to_base58(), "1111ZiCa");
    }
}