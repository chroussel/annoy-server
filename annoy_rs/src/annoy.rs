use super::native;
use super::vector;

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
        AnnoyIndexBuilder { dimension, raw }
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
        let result_vec = vector::IntVector::new();
        let distances_vec = vector::FloatVector::new();

        unsafe {
            native::rust_annoy_index_get_nns_by_item(
                self.raw,
                item,
                n,
                search_k.unwrap_or(-1),
                result_vec.raw(),
                distances_vec.raw(),
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
        let result_vec = vector::IntVector::new();
        let distances_vec = vector::FloatVector::new();

        unsafe {
            native::rust_annoy_index_get_nns_by_vector(
                self.raw,
                w.as_ptr(),
                n,
                search_k.unwrap_or(-1),
                result_vec.raw(),
                distances_vec.raw(),
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
