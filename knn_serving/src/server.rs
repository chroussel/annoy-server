use capnp_rpc::RpcSystem;
use futures::future;
use futures::future::Executor;
use futures::Future;
use futures::Stream;
use hyper::Server;
use knn_serving_api::service_capnp::knn_service;
use service::{Knn, KnnRPC, KnnService};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use tokio_core::net::TcpListener;
use tokio_core::reactor;
use tokio_core::reactor::Handle;
use tokio_current_thread;
use tokio_io::AsyncRead;
use tokio_threadpool::ThreadPool;

fn start_rpc(
    service: KnnRPC,
    addr: SocketAddr,
    handle: Handle,
) -> impl Future<Item = (), Error = ()> {
    let proxy = knn_service::ToClient::new(service).from_server::<capnp_rpc::Server>();
    let socket = TcpListener::bind(&addr, &handle).unwrap();
    info!("Start listening on {}", &addr);

    let f = socket
        .incoming()
        .for_each(move |(socket, _addr)| {
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
        })
        .map_err(|e| error!("Error in rpc system: {:?}", e));;
    f
}

pub fn start_http(knn: Arc<Knn>) -> impl Future<Item = (), Error = ()> {
    let addr = "127.0.0.1:3000".to_socket_addrs().unwrap().next().unwrap();

    let service = move || {
        let k = knn.clone();
        KnnService::new(k)
    };

    let server = Server::bind(&addr)
        .serve(service)
        .map_err(|e| println!("server error: {}", e));

    info!("Listening on {}", addr);
    server
}

fn main_loop(
    service: Arc<Knn>,
    addr: SocketAddr,
    handle: Handle,
) -> impl Future<Item = ((), ()), Error = ()> {
    future::lazy(move || {
        start_http(service.clone()).join(start_rpc(
            KnnRPC::new(service.clone()),
            addr,
            handle.clone(),
        ))
    })
}

pub fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} server HOST:PORT", args[0]);
        return;
    }

    let pool = ThreadPool::new();
    let service = Arc::new(Knn::new(pool));
    let addr = args[2]
        .to_socket_addrs()
        .unwrap()
        .next()
        .expect("could not parse address");
    let mut core = reactor::Core::new().unwrap();
    let handle = core.handle();

    core.run(main_loop(service, addr, handle)).unwrap();
}
