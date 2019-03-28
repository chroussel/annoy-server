use futures::Future;
use futures::Stream;
use hyper::Server;
use knn::Knn;
use service::KnnService;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

pub fn start_http(knn: Knn, http_addr: SocketAddr) -> impl Future<Item = (), Error = ()> {
    let server = Server::bind(&http_addr)
        .serve(move || {
            let k = knn.clone();
            KnnService::new(k)
        })
        .map_err(|e| info!("server error: {}", e));

    info!("Listening on {}", http_addr);
    server
}

pub fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} server HTTP_PORT", args[0]);
        return;
    }

    let http_port = args[2].parse::<u16>().expect("Unable to parse HTTP_PORT");
    let http_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), http_port);

    let service = Knn::new();
    tokio::run(start_http(service, http_addr));
}
