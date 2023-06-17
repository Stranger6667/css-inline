use core::hash::{BuildHasherDefault, Hasher};

pub(crate) type BuildNoHashHasher = BuildHasherDefault<NoHashHasher>;

#[derive(Default)]
pub(crate) struct NoHashHasher(u64);

impl Hasher for NoHashHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {
        panic!("Should not happen")
    }
    fn write_u8(&mut self, n: u8) {
        self.0 = u64::from(n)
    }
    fn write_u16(&mut self, n: u16) {
        self.0 = u64::from(n)
    }
    fn write_u32(&mut self, n: u32) {
        self.0 = u64::from(n)
    }
    fn write_u64(&mut self, n: u64) {
        self.0 = n
    }

    fn write_usize(&mut self, n: usize) {
        self.0 = n as u64
    }
    fn write_i8(&mut self, n: i8) {
        self.0 = n as u64
    }
    fn write_i16(&mut self, n: i16) {
        self.0 = n as u64
    }
    fn write_i32(&mut self, n: i32) {
        self.0 = n as u64
    }
    fn write_i64(&mut self, n: i64) {
        self.0 = n as u64
    }

    fn write_isize(&mut self, n: isize) {
        self.0 = n as u64
    }
}

#[cfg(test)]
mod tests {
    use super::NoHashHasher;
    use std::hash::Hasher;

    #[test]
    fn hasher() {
        let mut hasher = NoHashHasher::default();
        hasher.write_u8(42);
        assert_eq!(hasher.0, 42);
        hasher.write_u16(42);
        assert_eq!(hasher.0, 42);
        hasher.write_u32(42);
        assert_eq!(hasher.0, 42);
        hasher.write_u64(42);
        assert_eq!(hasher.0, 42);
        hasher.write_usize(42);
        assert_eq!(hasher.0, 42);
        hasher.write_i8(42);
        assert_eq!(hasher.0, 42);
        hasher.write_i16(42);
        assert_eq!(hasher.0, 42);
        hasher.write_i32(42);
        assert_eq!(hasher.0, 42);
        hasher.write_i64(42);
        assert_eq!(hasher.0, 42);
        hasher.write_isize(42);
        assert_eq!(hasher.0, 42);
    }

    #[test]
    #[should_panic]
    fn test_panic() {
        let mut hasher = NoHashHasher::default();
        hasher.write(b"a");
    }
}
