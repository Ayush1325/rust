use crate::{io, os::uefi};
use uefi_spec::{
    errors::StatusNullError,
    protocols::{simple_text_input, simple_text_output},
};

pub struct Stdin(());
pub struct Stdout(());
pub struct Stderr(());

impl Stdin {
    pub const fn new() -> Stdin {
        Stdin(())
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let buf_len = buf.len();
        let mut buf_count = 0;

        let st = unsafe {
            match uefi::get_system_table() {
                Ok(x) => x,
                Err(_) => {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "Global System Table"))
                }
            }
        };

        // Max 3 bytes can be required to store a ucs2 character as utf8
        while buf_len - buf_count >= 3 {
            let ch = match simple_text_input::read_key_stroke(st) {
                // Need to add check for non-printable keys
                Ok(x) => x.unicode_char,
                Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid Key")),
            };

            let l = utf16_to_utf8_char(ch, &mut buf[buf_count..]);
            buf_count += l;
        }

        Ok(buf_count)
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout(())
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let st = unsafe {
            match uefi::get_system_table() {
                Ok(x) => x,
                Err(_) => {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "Global System Table"))
                }
            }
        };

        let mut output = [0u16; 100];
        let count = utf8_to_utf16(buf, &mut output)?;
        output[count] = 0;

        match simple_text_output::output_string(st, &mut output) {
            Ok(()) => Ok(count),
            Err(e) => match e {
                StatusNullError::NullPtrError(s) => Err(io::Error::new(io::ErrorKind::Other, s)),
                StatusNullError::UefiWarning(u) | StatusNullError::UefiError(u) => {
                    Err(io::Error::new(io::ErrorKind::Other, u.to_string()))
                }
            },
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Stderr {
    pub const fn new() -> Stderr {
        Stderr(())
    }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub const STDIN_BUF_SIZE: usize = 0;

pub fn is_ebadf(_err: &io::Error) -> bool {
    true
}

pub fn panic_output() -> Option<Vec<u8>> {
    None
}

fn utf8_to_utf16(utf8_buf: &[u8], utf16_buf: &mut [u16]) -> io::Result<usize> {
    let utf8_buf_len = utf8_buf.len();
    let utf16_buf_len = utf16_buf.len();
    let mut utf8_buf_count = 0;
    let mut utf16_buf_count = 0;

    while utf8_buf_count < utf8_buf_len {
        match utf8_buf[utf8_buf_count] {
            0b0000_0000..0b0111_1111 => {
                // 1-byte
                utf16_buf[utf16_buf_count] = u16::from(utf8_buf[utf8_buf_count]);

                utf8_buf_count += 1;
            }
            0b1100_0000..0b1101_1111 => {
                // 2-byte
                if utf16_buf_count + 1 >= utf8_buf_len {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"));
                }
                let a = u16::from(utf8_buf[utf8_buf_count] & 0b0001_1111);
                let b = u16::from(utf8_buf[utf8_buf_count + 1] & 0b0011_1111);
                utf16_buf[utf16_buf_count] = a << 6 | b;

                utf8_buf_count += 2;
            }
            0b1110_0000..0b1110_1111 => {
                // 3-byte
                if utf16_buf_count + 2 >= utf8_buf_len {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"));
                }
                let a = u16::from(utf8_buf[utf8_buf_count] & 0b0000_1111);
                let b = u16::from(utf8_buf[utf8_buf_count + 1] & 0b0011_1111);
                let c = u16::from(utf8_buf[utf8_buf_count + 2] & 0b0011_1111);
                utf16_buf[utf16_buf_count] = a << 12 | b << 6 | c;
                utf8_buf_count += 3;
            }
            0b1111_0000..0b1111_0111 => {
                // 4-byte
                // We just print a restricted Character
                utf16_buf[utf16_buf_count] = 0xfffd;
                utf8_buf_count += 4;
            }
            _ => {
                // Invalid Data
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"));
            }
        }

        utf16_buf_count += 1;

        if utf16_buf_count >= utf16_buf_len {
            // The utf16 buffer is too small
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Buffer too small"));
        }
    }
    Ok(utf16_buf_count)
}

fn utf16_to_utf8_char(ch: u16, buf: &mut [u8]) -> usize {
    match ch {
        0b0000_0000_0000_0000..0b0000_0000_0111_1111 => {
            // 1-byte
            buf[0] = ch as u8;
            1
        }
        0b0000_0000_0111_1111..0b0000_0111_1111_1111 => {
            // 2-byte
            let a = ((ch & 0b0000_0111_1100_0000) >> 6) as u8;
            let b = (ch & 0b0000_0000_0011_1111) as u8;
            buf[0] = a | 0b1100_0000;
            buf[1] = b | 0b1000_0000;
            2
        }
        _ => {
            // 3-byte
            let a = ((ch & 0b1111_0000_0000_0000) >> 12) as u8;
            let b = ((ch & 0b0000_1111_1100_0000) >> 6) as u8;
            let c = (ch & 0b0000_0000_0011_1111) as u8;
            buf[0] = a | 0b1110_0000;
            buf[1] = b | 0b1000_0000;
            buf[2] = c | 0b1000_0000;
            3
        }
    }
}
