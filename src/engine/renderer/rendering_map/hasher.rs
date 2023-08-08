use std::hash::Hasher;

#[derive(Default)]
pub struct RenderingMapHasher {
    state: u64,
}

/// got from the anymap original crate : https://github.com/chris-morgan/anymap
impl Hasher for RenderingMapHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        // This expects to receive one and exactly one 64-bit value
        debug_assert!(bytes.len() == 8);
        unsafe {
            std::ptr::copy_nonoverlapping(&bytes[0] as *const u8 as *const u64, &mut self.state, 1)
        }
    }

    #[inline]
    fn finish(&self) -> u64 { self.state }
}

