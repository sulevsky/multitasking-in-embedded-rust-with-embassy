#![no_std]

pub mod statistics {
    use core::fmt::{self, Display};

    #[derive(Clone)]
    pub struct Statistics {
        min: u8,
        max: u8,
        mean: u8,
        count: u32,
    }

    impl Statistics {
        pub const fn new() -> Self {
            Self {
                min: u8::MAX,
                max: 0,
                mean: 0,
                count: 0,
            }
        }
        pub fn update_statistics(&mut self, element: u8) {
            self.min = u8::min(self.min, element);
            self.max = u8::max(self.max, element);
            self.mean = ((self.mean as u32 * self.count + element as u32) / (self.count + 1)) as u8;
            self.count += 1;
        }
    }

    impl Display for Statistics {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{{\"min\": {}, \"max\": {}, \"mean\": {}, \"count\": {}}}",
                self.min, self.max, self.mean, self.count
            )
        }
    }
}
pub mod writer {

    pub struct Buffer<const SIZE: usize> {
        buf: [u8; SIZE],
        len: usize,
        read_index: usize,
    }
    impl<const SIZE: usize> Buffer<SIZE> {
        pub const fn new() -> Self {
            Self {
                buf: [0u8; SIZE],
                len: 0,
                read_index: 0,
            }
        }
        pub fn read_next(&self) -> Option<u8> {
            if !self.is_read_finished() {
                Some(self.buf[self.read_index])
            } else {
                None
            }
        }
        pub fn mark_byte_as_read(&mut self) {
            if self.read_index < self.len {
                self.read_index += 1;
            }
            if self.read_index == self.len {
                self.len = 0;
                self.read_index = 0;
            }
        }
        pub fn is_read_finished(&self) -> bool {
            self.read_index == self.len
        }
    }
    impl<const SIZE: usize> core::fmt::Write for Buffer<SIZE> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            if bytes.len() > self.buf.len() - self.len {
                return Err(core::fmt::Error);
            }
            self.buf[self.len..self.len + bytes.len()].copy_from_slice(bytes);
            self.len += bytes.len();
            Ok(())
        }
    }
}
