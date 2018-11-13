use super::native;
use std::slice;

struct IntVector {
    raw: *mut native::i_vector,
}

impl IntVector {
    fn new() -> IntVector {
        let raw = unsafe { native::i_vector_init() };
        IntVector { raw }
    }

    fn data(&self) -> Vec<i32> {
        let raw_data = unsafe { native::i_vector_data(self.raw) };
        let raw_size = unsafe { native::i_vector_size(self.raw) };
        let raw_vec = unsafe { slice::from_raw_parts(raw_data, raw_size) };
        raw_vec.to_vec()
    }

    fn assign(&mut self, slice: &mut [i32]) {
        let raw_ptr = slice.as_mut_ptr();
        unsafe { native::i_vector_assign(self.raw, raw_ptr, slice.len()) };
    }
}

impl Drop for IntVector {
    fn drop(&mut self) {
        unsafe { native::i_vector_destroy(self.raw) };
    }
}

struct FloatVector {
    raw: *mut native::f_vector,
}

impl FloatVector {
    fn new() -> FloatVector {
        let raw = unsafe { native::f_vector_init() };
        FloatVector { raw }
    }

    fn data(&self) -> Vec<f32> {
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

pub struct AnnoyIndexBuilder {
    dimension: i32,
    raw: native::rust_annoy_index_t,
}

pub struct AnnoyIndex {
    dimension: i32,
    tree_count: Option<i32>,
    raw: native::rust_annoy_index_t,
}

impl AnnoyIndexBuilder {
    pub fn new(dimension: i32) -> AnnoyIndexBuilder {
        let raw = unsafe { native::rust_annoy_index_angular_init(dimension) };
        AnnoyIndexBuilder {
            dimension: dimension,
            raw,
        }
    }

    pub fn add_item(&mut self, item: i32, v: &[f32]) {
        unsafe { native::rust_annoy_index_add_item(self.raw, item, v.as_ptr()) };
    }

    pub fn build(self, n_tree: Option<i32>) -> AnnoyIndex {
        unsafe { native::rust_annoy_index_build(self.raw, n_tree.unwrap_or(-1)) };
        AnnoyIndex {
            dimension: self.dimension,
            raw: self.raw,
            tree_count: n_tree,
        }
    }

    pub fn dimension(&self) -> i32 {
        self.dimension
    }
}

impl AnnoyIndex {
    pub fn dimension(&self) -> i32 {
        self.dimension
    }

    pub fn tree_count(&self) -> Option<i32> {
        self.tree_count
    }

    pub fn get_nns_by_item(
        &self,
        item: i32,
        n: i32,
        search_k: Option<i32>,
    ) -> (Vec<i32>, Vec<f32>) {
        let result_vec = IntVector::new();
        let distances_vec = FloatVector::new();

        unsafe {
            native::rust_annoy_index_get_nns_by_item(
                self.raw,
                item,
                n,
                search_k.unwrap_or(-1),
                result_vec.raw,
                distances_vec.raw,
            )
        }

        (
            result_vec.data().to_owned(),
            distances_vec.data().to_owned(),
        )
    }

    pub fn get_nns_by_vector(
        &self,
        w: &[f32],
        n: i32,
        search_k: Option<i32>,
    ) -> (Vec<i32>, Vec<f32>) {
        let result_vec = IntVector::new();
        let distances_vec = FloatVector::new();

        unsafe {
            native::rust_annoy_index_get_nns_by_vector(
                self.raw,
                w.as_ptr(),
                n,
                search_k.unwrap_or(-1),
                result_vec.raw,
                distances_vec.raw,
            )
        };

        (
            result_vec.data().to_owned(),
            distances_vec.data().to_owned(),
        )
    }
}

impl Drop for AnnoyIndex {
    fn drop(&mut self) {
        unsafe { native::rust_annoy_index_destroy(self.raw) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_test() {
        let mut a = AnnoyIndexBuilder::new(3);
        a.add_item(0, &[1.0, 0.0, 0.0]);
        a.add_item(1, &[0.0, 1.0, 0.0]);
        a.add_item(2, &[0.0, 0.0, 1.0]);

        let index = a.build(None);

        println!("{:?}", index.get_nns_by_item(0, 100, None));
        println!("{:?}", index.get_nns_by_vector(&[1.0, 0.5, 0.5], 100, None));
    }
}
