use crate::ffi::{OsStr, OsString};
use crate::iter::FusedIterator;
use crate::sealed::Sealed;
use crate::str;
use crate::sys::os_str::Buf;
use crate::sys_common::{AsInner, FromInner};

const REPLACEMENT_CHARACTER_UCS2: u16 = 0xfffdu16;

#[unstable(feature = "uefi_std", issue = "none")]
pub trait OsStrExt: Sealed {
    fn to_ucs2<'a>(&'a self) -> EncodeUcs2<'a>;
}

impl OsStrExt for OsStr {
    fn to_ucs2<'a>(&'a self) -> EncodeUcs2<'a> {
        // This conversion should never fail since the underlying OsStr is UTF-8 encoded
        EncodeUcs2::from_str(self.to_str().unwrap())
    }
}

#[unstable(feature = "uefi_std", issue = "none")]
pub trait OsStringExt: Sealed
where
    Self: Sized,
{
    fn from_ucs2(ucs: &[u16]) -> Self;

    // For Null terminated UCS-2 Strings
    fn from_ucs2_null_termintated(ucs: &[u16]) -> Self {
        Self::from_ucs2(&ucs[..(ucs.len() - 1)])
    }
}

impl OsStringExt for OsString {
    fn from_ucs2(ucs: &[u16]) -> Self {
        // Min capacity(in case of all ASCII) is `ucs.len()`
        let mut buf = String::with_capacity(ucs.len());

        for i in ucs {
            let c = ucs2_to_utf8_char(*i);
            buf.push(c);
        }

        Self::from(buf)
    }
}

pub(crate) fn ucs2_to_utf8_char(ch: u16) -> char {
    match ch {
        0b0000_0000_0000_0000..0b0000_0000_0111_1111 => {
            // 1-byte

            unsafe { char::from_u32_unchecked(u32::from(ch)) }
        }
        0b0000_0000_0111_1111..0b0000_0111_1111_1111 => {
            // 2-byte
            let high = ((ch & 0b0000_0111_1100_0000) << 2) | 0b1100_0000_0000_0000;
            let low = (ch & 0b0000_0000_0011_1111) | 0b0000_0000_1000_0000;

            unsafe { char::from_u32_unchecked(u32::from(high | low)) }
        }
        _ => {
            // 3-byte
            let ch = ch as u32;
            let high = ((ch & 0b0000_0000_0000_0000_1111_0000_0000_0000) << 4)
                | 0b0000_0000_1110_0000_0000_0000_0000_0000;
            let mid = ((ch & 0b0000_0000_0000_0000_0000_1111_1100_0000) << 2)
                | 0b0000_0000_0000_0000_1000_0000_0000_0000;
            let low = (ch & 0b0000_0000_0000_0000_0000_0000_0011_1111)
                | 0b0000_0000_0000_0000_0000_0000_1000_0000;

            unsafe { char::from_u32_unchecked(u32::from(high | mid | low)) }
        }
    }
}

pub(crate) fn utf8_to_ucs2_char(ch: char) -> Option<u16> {
    let mut buf = [0u8; 4];
    ch.encode_utf8(&mut buf);

    match ch.len_utf8() {
        1 => Some(u16::from(buf[0])),
        2 => {
            let a = u16::from(buf[0] & 0b0001_1111);
            let b = u16::from(buf[1] & 0b0011_1111);
            Some(a << 6 | b)
        }
        3 => {
            let a = u16::from(buf[0] & 0b0000_1111);
            let b = u16::from(buf[1] & 0b0011_1111);
            let c = u16::from(buf[2] & 0b0011_1111);
            Some(a << 12 | b << 6 | c)
        }
        4 => None,
        _ => unreachable!(),
    }
}

pub struct Ucs2Char {
    pub unicode_char: u16,
    pub utf8_len: usize,
}

impl Ucs2Char {
    pub fn new(unicode_char: u16, utf8_len: usize) -> Self {
        Self { unicode_char, utf8_len }
    }
}

pub struct EncodeUcs2<'a> {
    utf8_buf: str::Chars<'a>,
}

impl<'a> EncodeUcs2<'a> {
    fn from_str(s: &'a str) -> Self {
        Self { utf8_buf: s.chars() }
    }

    // Returns error if slice of bytes is not valid UTF-8
    pub fn from_bytes(s: &'a [u8]) -> Result<Self, str::Utf8Error> {
        Ok(Self::from_str(str::from_utf8(s)?))
    }

    unsafe fn from_bytes_unchecked(s: &'a [u8]) -> Self {
        Self::from_str(unsafe { str::from_utf8_unchecked(s) })
    }
}

impl<'a> Iterator for EncodeUcs2<'a> {
    type Item = Ucs2Char;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ch) = self.utf8_buf.next() {
            match utf8_to_ucs2_char(ch) {
                Some(x) => Some(Ucs2Char::new(x, ch.len_utf8())),
                None => Some(Ucs2Char::new(REPLACEMENT_CHARACTER_UCS2, ch.len_utf8())),
            }
        } else {
            None
        }
    }
}

impl<'a> FusedIterator for EncodeUcs2<'a> {}
