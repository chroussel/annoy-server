use annoy_rs::annoy::Distance;
use annoy_rs::idmapping;
use capnp::message::{Builder, HeapAllocator};
use err::Error;
use evmap::{ReadHandle, WriteHandle};
use futures::{future, Future};
use knn_serving_api::service_capnp::{knn_request, knn_request_by_id, knn_response};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub type KnnMapRead = ReadHandle<String, Arc<idmapping::MappingIndex<i64>>>;
pub type KnnMapWrite = Arc<Mutex<WriteHandle<String, Arc<idmapping::MappingIndex<i64>>>>>;

#[derive(Clone)]
pub struct Knn {
    pub index_read: KnnMapRead,
    pub index_write: KnnMapWrite,
}

impl Knn {
    pub fn new() -> Knn {
        let (r, w) = evmap::new();
        Knn {
            index_read: r,
            index_write: Arc::new(Mutex::new(w)),
        }
    }

    const INDEX_FILE_NAME: &'static str = "index";
    const MAPPING_FILE_NAME: &'static str = "mapping";
    const DIMENSION_FILE_NAME: &'static str = "dimension";

    fn read_dimension_file<P: AsRef<Path>>(path: P) -> Result<i32, Error> {
        let path = path.as_ref();
        let mut file = File::open(path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(buf.parse::<i32>().map_err(|_| Error::ParsingError(buf))?)
    }

    pub fn load<P: AsRef<Path>>(
        index_write: KnnMapWrite,
        name: &str,
        path: P,
    ) -> Result<(), Error> {
        let path = path.as_ref().to_owned();
        let dimension = Knn::read_dimension_file(path.clone().join(Knn::DIMENSION_FILE_NAME))?;

        info!("Loading index {} from {}", name, path.display());

        let index = idmapping::MappingIndex::load(
            name,
            path.clone().join(Knn::INDEX_FILE_NAME),
            path.clone().join(Knn::MAPPING_FILE_NAME),
            dimension,
            Distance::Euclidean,
            true,
        )?;
        info!(
            "Index {} with {} items was loaded succesfully",
            name,
            index.len()
        );
        index_write
            .lock()
            .unwrap()
            .insert(name.to_owned(), Arc::new(index));
        index_write.lock().unwrap().refresh();
        Ok(())
    }

    pub fn search(
        index: Arc<idmapping::MappingIndex<i64>>,
        vector: Vec<f32>,
        k: i32,
        n: i32,
    ) -> impl Future<Item = (std::vec::Vec<i64>, std::vec::Vec<f32>), Error = Error> {
        debug!("New request: @{}, for {} item", k, n);
        if vector.len() != index.dimension() as usize {
            return future::err(Error::DimensionError(
                vector.len(),
                index.dimension() as usize,
            ));
        }
        let res = index.get_nns_by_vector(vector.as_slice(), n, Some(k));
        future::ok(res)
    }

    pub fn get_index<'a>(&self, name: &'a str) -> Result<Arc<idmapping::MappingIndex<i64>>, Error> {
        Knn::get_index2(self.index_read.clone(), name)
    }

    pub fn get_index2(
        map: KnnMapRead,
        name: &str,
    ) -> Result<Arc<idmapping::MappingIndex<i64>>, Error> {
        map.get_and(name, |v| v[0].clone())
            .ok_or_else(|| Error::NoIndexLoaded(name.to_owned()))
    }

    pub fn get_vector(index: &Arc<idmapping::MappingIndex<i64>>, id: i64) -> Option<Vec<f32>> {
        index.get_item_vector(id)
    }

    pub fn create_response_from_vectors(
        index: &Arc<idmapping::MappingIndex<i64>>,
        response_builder: knn_response::Builder,
        ids: &[i64],
        distances: &[f32],
    ) -> Result<(), Error> {
        let mut list: capnp::struct_list::Builder<knn_response::item::Owned> =
            response_builder.init_items(ids.len() as u32);
        for (i, elements) in ids.iter().enumerate() {
            let mut item: knn_response::item::Builder = list.reborrow().get(i as u32);
            item.set_id(*elements);
            /*let v = index.get_item_vector(*elements).unwrap();
            {
                let mut pv: capnp::primitive_list::Builder<f32> =
                    item.reborrow().init_vector(v.len() as u32);
                for (j, value) in v.into_iter().enumerate() {
                    pv.set(j as u32, value);
                }
            }*/
            item.set_distance(distances[i])
        }
        Ok(())
    }

    pub fn search2(
        index: Arc<idmapping::MappingIndex<i64>>,
        request: knn_request::Reader,
    ) -> Box<dyn Future<Item = Builder<HeapAllocator>, Error = Error> + Send> {
        let v: Vec<f32> = request.get_vector().unwrap().iter().collect();
        let k = request.get_search_k();
        let n = request.get_result_count();
        let index_copy = index.clone();
        let res = Knn::search(index, v, k, n).and_then(move |(r, d)| {
            let mut message = ::capnp::message::Builder::new_default();
            {
                let response: knn_response::Builder =
                    message.init_root::<knn_serving_api::service_capnp::knn_response::Builder>();
                Knn::create_response_from_vectors(
                    &index_copy,
                    response,
                    r.as_slice(),
                    d.as_slice(),
                )?;
            }
            Ok(message)
        });
        Box::new(res)
    }

    pub fn search_id(
        index: Arc<idmapping::MappingIndex<i64>>,
        request: knn_request_by_id::Reader,
    ) -> Box<dyn Future<Item = Builder<HeapAllocator>, Error = Error> + Send> {
        let pid = request.get_product_id();
        let v = Knn::get_vector(&index, pid);
        if v.is_none() {
            return Box::new(future::err(Error::NoProductVectorFound(pid)));
        }
        let v = v.unwrap();
        let k = request.get_search_k();
        let n = request.get_result_count();
        let index_copy = index.clone();
        let res = Knn::search(index, v, k, n).and_then(move |(r, d)| {
            let mut message = ::capnp::message::Builder::new_default();
            {
                let response: knn_response::Builder =
                    message.init_root::<knn_serving_api::service_capnp::knn_response::Builder>();
                Knn::create_response_from_vectors(
                    &index_copy,
                    response,
                    r.as_slice(),
                    d.as_slice(),
                )?;
            }
            Ok(message)
        });
        Box::new(res)
    }
}
