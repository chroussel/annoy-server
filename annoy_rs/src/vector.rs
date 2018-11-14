use super::native;
use std::slice;

pub struct IntVector {
    raw: *mut native::i_vector,
}

impl IntVector {
    pub fn new() -> IntVector {
        let raw = unsafe { native::i_vector_init() };
        IntVector { raw }
    }

    pub fn raw(&self) -> *mut native::i_vector {
        self.raw
    }

    pub fn data(&self) -> Vec<i32> {
        let raw_data = unsafe { native::i_vector_data(self.raw) };
        let raw_size = unsafe { native::i_vector_size(self.raw) };
        let raw_vec = unsafe { slice::from_raw_parts(raw_data, raw_size) };
        raw_vec.to_vec()
    }

    pub fn assign(&mut self, slice: &mut [i32]) {
        let raw_ptr = slice.as_mut_ptr();
        unsafe { native::i_vector_assign(self.raw, raw_ptr, slice.len()) };
    }
}

impl Drop for IntVector {
    fn drop(&mut self) {
        unsafe { native::i_vector_destroy(self.raw) };
    }
}

pub struct FloatVector {
    raw: *mut native::f_vector,
}

impl FloatVector {
    pub fn new() -> FloatVector {
        let raw = unsafe { native::f_vector_init() };
        FloatVector { raw }
    }

    pub fn raw(&self) -> *mut native::f_vector {
        self.raw
    }

    pub fn data(&self) -> Vec<f32> {
        let raw_data = unsafe { native::f_vector_data(self.raw) };
        let raw_size = unsafe { native::f_vector_size(self.raw) };
        let raw_vec = unsafe { slice::from_raw_parts(raw_data, raw_size) };
        raw_vec.to_vec()
    }

    fn assign(&mut self, slice: &mut [f32]) {
        let raw_ptr = slice.as_mut_ptr();
        unsafe { native::f_vector_assign(self.raw, raw_ptr, slice.len()) };
    }
}

impl Drop for FloatVector {
    fn drop(&mut self) {
        unsafe { native::f_vector_destroy(self.raw) };
    }
}
