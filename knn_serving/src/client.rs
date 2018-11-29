use capnp::capability::{Request, Response};
use capnp::{primitive_list, struct_list};
use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use futures::Future;
use knn_serving_api::service_capnp::{knn_request, knn_response, knn_service};
use std::net::ToSocketAddrs;
use tokio_core;
use tokio_io::AsyncRead;

pub fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} client HOST:PORT", args[0]);
        return;
    }

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let addr = args[2]
        .to_socket_addrs()
        .unwrap()
        .next()
        .expect("could not parse address");
    let stream: tokio_core::net::TcpStream = core
        .run(tokio_core::net::TcpStream::connect(&addr, &handle))
        .unwrap();

    stream.set_nodelay(true).unwrap();

    let (reader, writer) = stream.split();
    let rpc = Box::new(twoparty::VatNetwork::new(
        reader,
        writer,
        rpc_twoparty_capnp::Side::Client,
        Default::default(),
    ));

    let mut rpc_system = RpcSystem::new(rpc, None);

    let proxy: knn_service::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);
    handle.spawn(rpc_system.map_err(|_e| ()));

    let mut req: Request<knn_service::search_params::Owned, knn_service::search_results::Owned> =
        proxy.search_request();

    {
        let mut builder: knn_request::Builder = req.get().init_request();
        builder.set_algorithm(knn_request::Algorithm::Annoy);
        builder.set_result_count(10);
        builder.set_search_k(100);
        let mut vector_builder: primitive_list::Builder<f32> = builder.init_vector(20);
        let vec = vec![
            0.1, 0.3, 0.4, 0.1, 0.5, 1.2, 0.6, 0.5, 0.9, 0.12, 0.1, 0.3, 0.4, 0.1, 0.5, 1.2, 0.6,
            0.5, 0.9, 0.12,
        ];

        for (i, v) in vec.into_iter().enumerate() {
            vector_builder.set(i as u32, v);
        }
    }

    println!("Sending request");

    let response: Response<knn_service::search_results::Owned> =
        core.run(req.send().promise).unwrap();
    let response: knn_service::search_results::Reader = response.get().unwrap();
    let response: knn_response::Reader = response.get_response().unwrap();

    let _rc = response.get_result_count();
    let items: struct_list::Reader<knn_response::item::Owned> = response.get_items().unwrap();

    for item in items.iter() {
        let a: knn_response::item::Reader = item;
        let v: Vec<_> = a.get_vector().unwrap().iter().collect();
        println!(
            "Got id: {}, vector: {:?} distance: {}",
            a.get_id(),
            v,
            a.get_distance()
        );
    }
}
