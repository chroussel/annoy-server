use super::native;
use super::vector;
use err;
use std::ffi::CString;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub enum Distance {
    Angular,
    Euclidean,
    Manhattan,
}

pub struct AnnoyIndexRaw(native::rust_annoy_index_t);

impl AnnoyIndexRaw {
    fn new(dimension: i32, dis: &Distance) -> AnnoyIndexRaw {
        let raw = match dis {
            Distance::Angular => unsafe { native::rust_annoy_index_angular_init(dimension) },
            Distance::Euclidean => unsafe { native::rust_annoy_index_euclidian_init(dimension) },
            Distance::Manhattan => unsafe { native::rust_annoy_index_manhattan_init(dimension) },
        };

        AnnoyIndexRaw(raw)
    }
}

impl Drop for AnnoyIndexRaw {
    fn drop(&mut self) {
        unsafe { native::rust_annoy_index_destroy(self.0) };
    }
}

pub struct AnnoyIndexBuilder {
    dimension: i32,
    distance: Distance,
    raw: AnnoyIndexRaw,
    item_count: i32,
}

pub struct AnnoyIndex {
    dimension: i32,
    distance: Distance,
    tree_count: Option<i32>,
    raw: AnnoyIndexRaw,
}

impl AnnoyIndexBuilder {
    /// Create a builder for index with vector of dimension [dimension]
    /// ```
    /// use annoy_rs::annoy::*;
    /// let mut builder = AnnoyIndexBuilder::new(10, Distance::Angular);
    /// ```

    pub fn new(dimension: i32, distance: Distance) -> AnnoyIndexBuilder {
        let raw = AnnoyIndexRaw::new(dimension, &distance);
        AnnoyIndexBuilder {
            dimension,
            distance,
            raw,
            item_count: -1,
        }
    }

    /// Add a vector to the index
    /// returns item index
    /// ```
    /// use annoy_rs::annoy::*;
    /// let mut builder = AnnoyIndexBuilder::new(2, Distance::Angular);
    /// builder.add_item(&[0.0, 1.0]);
    /// let item_index = builder.add_item(&[0.0, 1.0]);
    /// assert_eq!(item_index, 1);
    /// ```
    pub fn add_item(&mut self, v: &[f32]) -> i32 {
        self.item_count += 1;
        unsafe { native::rust_annoy_index_add_item(self.raw.0, self.item_count, v.as_ptr()) };
        self.item_count
    }

    /// Build the index. Internally it create [n_tree] trees
    /// If None is specified it creates 2 * [item_count] trees
    ///
    /// ```
    /// use annoy_rs::annoy::*;
    /// let mut builder = AnnoyIndexBuilder::new(2, Distance::Angular);
    /// builder.add_item(&[0.0, 1.0]);
    /// builder.add_item(&[0.0, 1.0]);
    /// let index = builder.build(Some(2));
    /// ```
    pub fn build(self, n_tree: Option<i32>) -> AnnoyIndex {
        unsafe { native::rust_annoy_index_build(self.raw.0, n_tree.unwrap_or(-1)) };
        AnnoyIndex {
            distance: self.distance,
            dimension: self.dimension,
            raw: self.raw,
            tree_count: n_tree,
        }
    }

    /// Return the dimension of the index which was given at creation
    /// ```
    /// use annoy_rs::annoy::*;
    /// let mut builder = AnnoyIndexBuilder::new(2, Distance::Angular);
    /// assert_eq!(builder.dimension(), 2)
    /// ```
    pub fn dimension(&self) -> i32 {
        self.dimension
    }

    pub fn distance(&self) -> &Distance {
        &self.distance
    }
}

impl AnnoyIndex {
    /// Return the dimension used to build the index
    /// ```
    /// use annoy_rs::annoy::*;
    /// let mut builder = AnnoyIndexBuilder::new(2, Distance::Angular);
    /// let index = builder.build(None);
    /// assert_eq!(index.dimension(), 2)
    /// ```
    pub fn dimension(&self) -> i32 {
        self.dimension
    }

    /// Return the distance function used to compute the index
    /// ```
    /// use annoy_rs::annoy::*;
    /// let mut builder = AnnoyIndexBuilder::new(2, Distance::Angular);
    /// let index = builder.build(None);
    /// assert_eq!(index.distance(), &Distance::Angular)
    /// ```
    pub fn distance(&self) -> &Distance {
        &self.distance
    }

    /// Return the number of tree used to build the index
    /// ```
    /// use annoy_rs::annoy::*;
    /// let mut builder = AnnoyIndexBuilder::new(2,  Distance::Angular);
    /// let index = builder.build(Some(123));
    /// assert_eq!(index.tree_count(), Some(123))
    /// ```
    pub fn tree_count(&self) -> Option<i32> {
        self.tree_count
    }

    /// Return the [n] closer item to item index [item] searching [search_k] tree
    /// When using None for search_k it uses [n_trees * n]
    /// It return 2 array containing results and distance to the item.
    ///
    /// ```
    /// use annoy_rs::annoy::*;
    /// let mut builder = AnnoyIndexBuilder::new(2, Distance::Angular);
    /// builder.add_item(&[1.0, 0.0, 0.0]);
    /// builder.add_item(&[0.0, 1.0, 0.0]);
    /// builder.add_item(&[0.0, 0.0, 1.0]);
    /// let index = builder.build(None);
    /// let (results, distances) = index.get_nns_by_item(1, 2, Some(3));
    /// assert_eq!(results.len(), 2);
    /// assert_eq!(distances.len(), 2);
    /// ```
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
                self.raw.0,
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

    /// Return the [n] closer item to item index [item] searching [search_k] tree
    /// When using None for search_k it uses [n_trees * n]
    /// It return 2 array containing results and distance to the item.
    ///
    /// ```
    /// use annoy_rs::annoy::*;
    /// let mut builder = AnnoyIndexBuilder::new(2, Distance::Angular);
    /// builder.add_item(&[1.0, 0.0, 0.0]);
    /// builder.add_item(&[0.0, 1.0, 0.0]);
    /// builder.add_item(&[0.0, 0.0, 1.0]);
    /// let index = builder.build(None);
    /// let (results, distances) = index.get_nns_by_vector(&[1.0, 0.5, 0.5], 2, Some(2));
    /// assert_eq!(results.len(), 2);
    /// assert_eq!(distances.len(), 2);
    /// ```
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
                self.raw.0,
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

    pub fn save2<P: AsRef<Path>>(&self, path: P, load_into_ram: bool) -> Result<(), err::Error> {
        let path_str = path
            .as_ref()
            .as_os_str()
            .to_str()
            .ok_or(err::Error::InvalidPath)?;

        let cs = CString::new(path_str).unwrap();
        unsafe { native::rust_annoy_index_save(self.raw.0, cs.as_ptr(), load_into_ram) };
        Ok(())
    }

    pub fn save(&self, path: PathBuf) -> Result<(), err::Error> {
        self.save2(path, false)
    }

    pub fn load2<P: AsRef<Path>>(&self, path: P, load_into_ram: bool) -> Result<(), err::Error> {
        let path_str = path
            .as_ref()
            .as_os_str()
            .to_str()
            .ok_or(err::Error::InvalidPath)?;

        let cs = CString::new(path_str).unwrap();
        unsafe { native::rust_annoy_index_load(self.raw.0, cs.as_ptr(), load_into_ram) };
        Ok(())
    }

    pub fn load(&self, path: PathBuf) -> Result<(), err::Error> {
        self.load2(path, false)
    }

    pub fn len(&self) -> i32 {
        unsafe { native::rust_annoy_index_get_n_item(self.raw.0) }
    }

    pub fn is_empty(&self) -> bool {
        self.len() != 0
    }

    pub fn get_item(&self, item: i32) -> Option<Vec<f32>> {
        if item >= self.len() {
            return None;
        }
        let mut vec: Vec<f32> = Vec::with_capacity(self.dimension() as usize);
        let ptr = vec.as_mut_ptr();

        unsafe {
            std::mem::forget(vec);
            native::rust_annoy_index_get_item(self.raw.0, item, ptr);
        };

        let new_vec = unsafe {
            Vec::from_raw_parts(ptr, self.dimension() as usize, self.dimension() as usize)
        };
        Some(new_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::Standard;
    use rand::prelude::*;
    use rand::Rng;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::time::SystemTime;

    #[test]
    fn simple_test() {
        let mut a = AnnoyIndexBuilder::new(3, Distance::Angular);
        a.add_item(&[1.0, 0.0, 0.0]);
        a.add_item(&[0.0, 1.0, 0.0]);
        a.add_item(&[0.0, 0.0, 1.0]);

        let index = a.build(None);

        println!("{:?}", index.get_nns_by_item(0, 100, None));
        println!("{:?}", index.get_nns_by_vector(&[1.0, 0.5, 0.5], 100, None));
    }

    #[test]
    fn mmap_test() {
        let mut a = AnnoyIndexBuilder::new(3, Distance::Angular);
        a.add_item(&[1.0, 0.0, 0.0]);
        a.add_item(&[0.0, 1.0, 0.0]);
        a.add_item(&[0.0, 0.0, 1.0]);

        let index = a.build(None);

        index.save(PathBuf::from("test.tree")).unwrap();

        let index2 = AnnoyIndexBuilder::new(3, Distance::Angular).build(None);
        index2.load(PathBuf::from("test.tree")).unwrap();

        println!("{:?}", index2.get_nns_by_item(0, 2, None));
        println!("{:?}", index2.get_nns_by_vector(&[1.0, 0.5, 0.5], 2, None));
    }

    #[test]
    fn get_n_item_test() {
        let mut a = AnnoyIndexBuilder::new(3, Distance::Angular);
        let count = 1123;
        for _i in 0..count {
            a.add_item(&[1.0, 0.0, 0.0]);
        }
        let index = a.build(None);
        assert_eq!((index.len()), count)
    }

    #[test]
    fn get_item_test() {
        let mut a = AnnoyIndexBuilder::new(3, Distance::Angular);
        a.add_item(&[1.0, 0.0, 0.0]);
        a.add_item(&[0.0, 1.0, 0.0]);
        a.add_item(&[0.0, 0.0, 1.0]);
        let index = a.build(None);

        assert_eq!(index.get_item(1), Some(vec![0.0, 1.0, 0.0]));
        assert_eq!(index.get_item(3), None);
        assert_eq!(index.get_item(5), None);
    }

    struct A {
        x: i32,
    }

    #[test]
    fn precision_test() {
        let n: i32 = 1000;
        const F: usize = 40;

        let mut rng = thread_rng();
        let mut index_builder = AnnoyIndexBuilder::new(F as i32, Distance::Angular);

        for _i in 0..n {
            let mut arr: Vec<f32> = rng.sample_iter(&Standard).take(F).collect();
            index_builder.add_item(arr.as_slice());
        }

        let index = index_builder.build(Some(400 as i32));
        let limits = &[20000];
        let k = 10;
        let mut prec_sum = HashMap::new();
        let prec_n = 1000;
        let mut time_sum = HashMap::new();
        for limit in limits.iter() {
            prec_sum.insert(limit, 0.0);
            time_sum.insert(limit, 0);
        }
        for _i in 0..prec_n {
            let j = rng.gen_range(0, n);
            let (closest, _) = index.get_nns_by_item(j, k, Some(n));
            let closest_set: HashSet<_> = closest.iter().collect();
            for limit in limits.iter() {
                let t0 = SystemTime::now();
                let (toplist, _) = index.get_nns_by_item(j, k, Some(*limit));
                let T = t0.elapsed().unwrap();
                let top_list_set: HashSet<_> = toplist.iter().collect();

                let found = closest_set.intersection(&top_list_set).count();
                let hitrate = 1.0 * (found as f32) / (k as f32);

                if let Some(value) = prec_sum.get_mut(limit) {
                    *value += hitrate;
                }

                if let Some(value) = time_sum.get_mut(limit) {
                    *value += T.as_nanos();
                }
            }
        }

        for limit in limits.iter() {
            let prec = 100.0 * prec_sum.get(limit).unwrap() / (prec_n + 1) as f32;
            let avg_time = time_sum.get(limit).unwrap() / (prec_n + 1);
            println!(
                "limit: {:>6} - precision: {:.6}% - avg time: {} ns",
                limit, prec, avg_time
            )
        }
    }
}
