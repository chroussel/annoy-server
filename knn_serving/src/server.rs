use annoy_rs::annoy::Distance;
use annoy_rs::idmapping;
use capnp::capability::Promise;
use capnp::text;
use capnp::Error;
use capnp_rpc::RpcSystem;
use futures::sync::oneshot;
use futures::Stream;
use futures::{Async, Future};
use service_capnp::knn_request;
use service_capnp::knn_response;
use service_capnp::knn_service;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::net::ToSocketAddrs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio_core::net::TcpListener;
use tokio_core::reactor;
use tokio_io::AsyncRead;
use tokio_threadpool::{Sender, ThreadPool};
use util::*;

struct KNNService {
    pool: ThreadPool,
    handle: reactor::Handle,
    indexes: HashMap<String, Arc<idmapping::MappingIndex<i64>>>,
}

impl<'a> KNNService {
    pub fn new(handle: reactor::Handle, pool: ThreadPool) -> KNNService {
        KNNService {
            pool,
            handle,
            indexes: HashMap::default(),
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
        Ok(buf
            .parse::<i32>()
            .map_err(|e| Error::failed(format!("Error while parsing file: {:?}", e)))?)
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

impl Future for AnnoyFuture {
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

fn compute_async(
    pool: &Sender,
    index: Arc<idmapping::MappingIndex<i64>>,
    vector: Vec<f32>,
    n: i32,
    k: i32,
) -> impl Future<Item = (Vec<i64>, Vec<f32>), Error = Error> {
    let (tx, rx) = oneshot::channel::<(Vec<i64>, Vec<f32>)>();
    let p: &Sender = pool;
    p.spawn(
        AnnoyFuture::new(index, vector, n, k).and_then(move |(r, d)| {
            tx.send((r, d))
                .unwrap_or_else(|e| warn!("Failed to push data to channel {:?}", e));
            Ok(())
        }),
    )
    .unwrap_or_else(|e| warn!("Failed to spawn computation: {:?}", e));

    rx.map_err(|_e| Error::failed(String::from("Future has bee, cancelled")))
}

impl knn_service::Server for KNNService {
    fn search(
        &mut self,
        params: knn_service::SearchParams,
        mut results: knn_service::SearchResults,
    ) -> Promise<(), Error> {
        let request: knn_request::Reader = params.get().unwrap().get_request().unwrap();
        let name = request.get_index_name().unwrap();
        let v: Vec<f32> = request.get_vector().unwrap().iter().collect();
        let k = request.get_search_k();
        let n = request.get_result_count();

        let index = pry!(self
            .indexes
            .get(name)
            .ok_or_else(|| Error::failed(format!("Can't find index named {}", name))));
        let index = index.clone();
        if v.len() != index.dimension() as usize {
            return Promise::err(Error::failed(format!(
                "Vector has incorrect size ({}) for given index ({})",
                v.len(),
                index.dimension()
            )));
        }

        let poll = compute_async(self.pool.sender(), index.clone(), v, n, k);

        let result = poll.and_then(move |(r, d)| {
            let builder: knn_service::search_results::Builder = results.get();

            let response: knn_response::Builder = builder.init_response();
            create_response_from_vectors(index, response, r.as_slice(), d.as_slice());
            Ok(())
        });
        Promise::from_future(result)
    }

    fn load(
        &mut self,
        params: knn_service::LoadParams,
        _results: knn_service::LoadResults,
    ) -> Promise<(), Error> {
        let params = params.get().unwrap();
        let name: text::Reader = params.get_index_name().unwrap();
        let index_path: text::Reader = params.get_index_path().unwrap();
        let path: PathBuf = PathBuf::from(index_path);

        let dimension = pry!(KNNService::read_dimension_file(
            path.clone().join(KNNService::DIMENSION_FILE_NAME)
        ));

        info!("Loading index {} from {}", name, path.display());

        let index = pry!(idmapping::MappingIndex::load(
            path.clone().join(KNNService::INDEX_FILE_NAME),
            path.clone().join(KNNService::MAPPING_FILE_NAME),
            dimension,
            Distance::Angular,
            true,
        )
        .map_err(capnp_error_from_err));

        self.indexes.insert(name.to_owned(), Arc::new(index));

        Promise::ok(())
    }
}

pub fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} server HOST:PORT", args[0]);
        return;
    }

    let mut core = reactor::Core::new().unwrap();
    let handle = core.handle();
    let pool = ThreadPool::new();

    let addr = args[2]
        .to_socket_addrs()
        .unwrap()
        .next()
        .expect("could not parse address");

    let socket = TcpListener::bind(&addr, &handle).unwrap();
    info!("Start listening on {}", args[2]);
    let proxy = knn_service::ToClient::new(KNNService::new(handle.clone(), pool))
        .from_server::<capnp_rpc::Server>();

    let done = socket.incoming().for_each(move |(socket, _addr)| {
        socket.set_nodelay(true)?;
        let (reader, writer) = socket.split();

        let network = ::capnp_rpc::twoparty::VatNetwork::new(
            reader,
            writer,
            ::capnp_rpc::rpc_twoparty_capnp::Side::Server,
            Default::default(),
        );

        let rpc_sys = RpcSystem::new(Box::new(network), Some(proxy.clone().client))
            .map_err(|e| error!("Error in rpc system: {:?}", e));

        let handle1 = handle.clone();
        handle1.spawn(rpc_sys);
        Ok(())
    });
    core.run(done).unwrap();
}
