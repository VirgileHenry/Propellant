pub trait ToBuffer {
    fn to_buffer(&self, buffer: &mut [u8]);
}

impl<T> ToBuffer for T {
    fn to_buffer(&self, buffer: &mut [u8]) {
        let size = std::mem::size_of::<T>();
        let ptr = self as *const T as *const u8;
        unsafe {
            std::ptr::copy_nonoverlapping(ptr, buffer.as_mut_ptr(), size);
        }
    }
}