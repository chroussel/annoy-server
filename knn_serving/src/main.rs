extern crate annoy_rs;
extern crate capnp;
extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate futures;
extern crate bytes;
extern crate hyper;
extern crate knn_serving_api;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate evmap;
extern crate serde_json;
extern crate tokio;

mod err;
mod knn;
mod server;
mod service;
mod util;

//mod service_capnp;

fn main() {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::try_init_from_env(env)
        .unwrap_or_else(|e| print!("Failed to initialize logger :{:?}", e));
    server::main();
}
