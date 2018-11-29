use annoy_rs::annoy::Distance;
use annoy_rs::idmapping;
use capnp::capability::Promise;
use capnp::message::{Builder, HeapAllocator};
use capnp::serialize_packed;
use capnp::text;
use err;
use err::Error;
use futures::future;
use futures::future::Either;
use futures::prelude::*;
use futures::sync::oneshot;
use futures::Async;
use futures::Future;
use futures::Stream;
use hyper::service::Service;
use hyper::{Body, Method, Request, Response, StatusCode};
use knn_serving_api::service_capnp::{knn_request, knn_response, knn_service};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio_threadpool::{Sender, ThreadPool};
use util;

macro_rules! fry {
    ($expr: expr) => {
        match $expr {
            Ok(value) => value,
            Err(error) => return Either::B(future::err(Error::from(error))),
        }
    };
}

type KnnMap = Arc<RwLock<HashMap<String, Arc<idmapping::MappingIndex<i64>>>>>;

pub struct Knn {
    pool: ThreadPool,
    indexes: KnnMap,
}

impl Knn {
    pub fn new(pool: ThreadPool) -> Knn {
        Knn {
            pool,
            indexes: Arc::new(RwLock::new(HashMap::default())),
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

    fn load<P: AsRef<Path>>(&self, name: &str, path: P) -> Result<(), Error> {
        let path = path.as_ref().to_owned();
        let dimension = Knn::read_dimension_file(path.clone().join(Knn::DIMENSION_FILE_NAME))?;

        info!("Loading index {} from {}", name, path.display());

        let index = idmapping::MappingIndex::load(
            path.clone().join(Knn::INDEX_FILE_NAME),
            path.clone().join(Knn::MAPPING_FILE_NAME),
            dimension,
            Distance::Angular,
            true,
        )?;
        self.indexes
            .write()
            .unwrap()
            .insert(name.to_owned(), Arc::new(index));
        Ok(())
    }

    fn search(
        sender: &Sender,
        index: Arc<idmapping::MappingIndex<i64>>,
        vector: Vec<f32>,
        k: i32,
        n: i32,
    ) -> Box<dyn Future<Item = (std::vec::Vec<i64>, std::vec::Vec<f32>), Error = Error> + Send>
    {
        if vector.len() != index.dimension() as usize {
            return Box::new(futures::future::err(Error::DimensionError(
                vector.len(),
                index.dimension() as usize,
            )));
        }

        let (tx, rx) = oneshot::channel::<(Vec<i64>, Vec<f32>)>();
        sender
            .spawn(
                AnnoyFuture::new(index, vector, n, k).and_then(move |(r, d)| {
                    tx.send((r, d))
                        .unwrap_or_else(|e| warn!("Failed to push data to channel {:?}", e));
                    Ok(())
                }),
            )
            .unwrap_or_else(|e| warn!("Failed to spawn computation: {:?}", e));

        let res = rx.map_err(|_e| Error::CancelledFuture);
        Box::new(res)
    }

    fn get_index<'a>(&self, name: &'a str) -> Result<Arc<idmapping::MappingIndex<i64>>, Error> {
        Knn::get_index2(&self.indexes, name)
    }

    fn get_index2(map: &KnnMap, name: &str) -> Result<Arc<idmapping::MappingIndex<i64>>, Error> {
        map.read()
            .unwrap()
            .get(name)
            .ok_or_else(|| Error::NoIndexLoaded(name.to_owned()))
            .map(|m| m.clone())
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
            let v = index.get_item_vector(*elements).unwrap();
            {
                let mut pv: capnp::primitive_list::Builder<f32> =
                    item.reborrow().init_vector(v.len() as u32);
                for (j, value) in v.into_iter().enumerate() {
                    pv.set(j as u32, value);
                }
            }
            item.set_distance(distances[i])
        }
        Ok(())
    }

    fn search2(
        sender: &Sender,
        index: Arc<idmapping::MappingIndex<i64>>,
        request: knn_serving_api::service_capnp::knn_request::Reader,
    ) -> Box<dyn Future<Item = Builder<HeapAllocator>, Error = Error> + Send> {
        let v: Vec<f32> = request.get_vector().unwrap().iter().collect();
        let k = request.get_search_k();
        let n = request.get_result_count();
        let index_copy = index.clone();
        let res = Knn::search(sender, index, v, k, n).and_then(move |(r, d)| {
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

struct AnnoyFuture {
    index: Arc<idmapping::MappingIndex<i64>>,
    vector: Vec<f32>,
    n: i32,
    k: i32,
}

impl AnnoyFuture {
    fn new(
        index: Arc<idmapping::MappingIndex<i64>>,
        vector: Vec<f32>,
        n: i32,
        k: i32,
    ) -> AnnoyFuture {
        AnnoyFuture {
            index,
            vector,
            n,
            k,
        }
    }
}

impl<'a> Future for AnnoyFuture {
    type Item = (Vec<i64>, Vec<f32>);
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        tokio_threadpool::blocking(|| {
            self.index
                .get_nns_by_vector(self.vector.as_slice(), self.n, Some(self.k))
        })
        .map_err(|e| error!("Error during future creation: {:?}", e))
    }
}

fn search(
    req: Request<Body>,
    hashmap: KnnMap,
    sender: Sender,
) -> Box<Future<Item = Response<Body>, Error = Error> + Send> {
    let body = req.into_body();
    let s = body
        .concat2()
        .into_future()
        .map_err(Error::from)
        .and_then(|buf| {
            Ok(try!(serialize_packed::read_message(
                &mut buf.as_ref(),
                ::capnp::message::ReaderOptions::default(),
            )))
        })
        .and_then(move |message_reader| {
            let request =
                fry!(message_reader
                    .get_root::<knn_serving_api::service_capnp::knn_request::Reader>());
            let name = fry!(request.get_index_name());
            let index = fry!(Knn::get_index2(&hashmap, name));
            let output = Knn::search2(&sender, index, request).and_then(move |builder| {
                let mut buffer = Vec::with_capacity(256);
                serialize_packed::write_message(&mut buffer, &builder)?;
                Ok(Response::new(Body::from(buffer)))
            });
            Either::A(output)
        });

    Box::new(s)
}

pub struct KnnRPC {
    state: Arc<Knn>,
}

impl KnnRPC {
    pub fn new(state: Arc<Knn>) -> KnnRPC {
        KnnRPC { state }
    }
}

impl knn_service::Server for KnnRPC {
    fn search(
        &mut self,
        params: knn_service::SearchParams,
        mut results: knn_service::SearchResults,
    ) -> Promise<(), capnp::Error> {
        let request: knn_request::Reader = params.get().unwrap().get_request().unwrap();
        let index = pry!(self
            .state
            .get_index(request.get_index_name().unwrap())
            .map_err(util::capnp_error_from_err));
        let sender = self.state.pool.sender();
        let res = Knn::search2(sender, index.clone(), request)
            .and_then(move |message_builder| {
                let mut builder = results.get();
                let reader = message_builder.into_reader();
                let response = reader.get_root::<knn_response::Reader>()?;
                builder.set_response(response)?;
                Ok(())
            })
            .map_err(util::capnp_error_from_err);

        Promise::from_future(res)
    }

    fn load(
        &mut self,
        params: knn_service::LoadParams,
        _results: knn_service::LoadResults,
    ) -> Promise<(), capnp::Error> {
        let params = params.get().unwrap();
        let name: text::Reader = params.get_index_name().unwrap();
        let index_path: text::Reader = params.get_index_path().unwrap();
        let path: PathBuf = PathBuf::from(index_path);

        let f = self
            .state
            .load(name, path)
            .map_err(util::capnp_error_from_err);
        Promise::from_future(f.into_future())
    }
}

pub struct KnnService {
    state: Arc<Knn>,
}

impl KnnService {
    pub fn new(state: Arc<Knn>) -> KnnService {
        KnnService { state }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct LoadRequest {
    pub index_name: String,
    pub path: String,
}

impl Service for KnnService {
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = Error;
    type Future = Box<dyn Future<Item = Response<Self::ResBody>, Error = Error> + Send>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        match (req.method(), req.uri().path()) {
            (&Method::POST, "/search") => {
                let hashmap = self.state.indexes.clone();
                let sender = self.state.pool.sender().clone();
                Box::new(search(req, hashmap, sender))
            }
            (&Method::POST, "/load") => {
                let state = self.state.clone();
                let f = req
                    .into_body()
                    .concat2()
                    .map_err(Error::from)
                    .and_then(
                        move |buf| match serde_json::from_slice::<LoadRequest>(&buf) {
                            Err(e) => Err(Error::from(e)),
                            Ok(loadr) => match state.load(&loadr.index_name, loadr.path) {
                                Err(e) => Err(e),
                                Ok(_) => Response::builder()
                                    .status(StatusCode::OK)
                                    .body(Body::empty())
                                    .map_err(Error::from),
                            },
                        },
                    );
                Box::new(f)
            }
            _ => Box::new(future::ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap(),
            )),
        }
    }
}

impl futures::future::IntoFuture for KnnService {
    type Future = future::FutureResult<Self::Item, Self::Error>;
    type Item = Self;
    type Error = err::Error;

    fn into_future(self) -> Self::Future {
        future::ok(self)
    }
}
