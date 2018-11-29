extern crate annoy_rs;
extern crate capnp;
extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate capnp_rpc;
extern crate bytes;
extern crate hyper;
extern crate knn_serving_api;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_curl;
extern crate tokio_current_thread;
extern crate tokio_executor;
extern crate tokio_io;
extern crate tokio_threadpool;

mod client;
mod err;
mod server;
mod service;
mod util;

//mod service_capnp;

fn main() {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::try_init_from_env(env)
        .unwrap_or_else(|e| print!("Failed to initialize logger :{:?}", e));

    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 {
        match &args[1][..] {
            "server" => return server::main(),
            "client" => return client::main(),
            _ => println!("Argument missing"),
        }
    }
}
