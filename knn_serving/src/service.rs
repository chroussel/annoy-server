use annoy_rs::annoy::Distance;
use annoy_rs::idmapping;
use capnp::capability::Promise;
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
use knn::{Knn, KnnMapRead};
use knn_serving_api::service_capnp::{knn_request, knn_request_by_id, knn_response, knn_service};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use util;

macro_rules! fry {
    ($expr: expr) => {
        match $expr {
            Ok(value) => value,
            Err(error) => return Either::B(future::err(Error::from(error))),
        }
    };
}

fn search(
    req: Request<Body>,
    hashmap: KnnMapRead,
) -> Box<Future<Item = Response<Body>, Error = Error> + Send> {
    let body = req.into_body();
    let s = body
        .concat2()
        .into_future()
        .map_err(Error::from)
        .and_then(|buf| {
            debug!("Deserializing message");
            serialize_packed::read_message(
                &mut buf.as_ref(),
                ::capnp::message::ReaderOptions::default(),
            )
            .map_err(Error::from)
        })
        .and_then(move |message_reader| {
            debug!("Sending to Knn service");
            let request = fry!(message_reader.get_root::<knn_request::Reader>());
            let name = fry!(request.get_index_name());
            let index = fry!(Knn::get_index2(hashmap.clone(), name));
            debug!("Searching in index: {}", name);
            Either::A(Knn::search2(index, request))
        })
        .and_then(move |builder| {
            debug!("Builing Response");
            let mut buffer = Vec::with_capacity(256);
            serialize_packed::write_message(&mut buffer, &builder)?;
            Ok(Response::new(Body::from(buffer)))
        });

    Box::new(s)
}

fn search2(
    req: Request<Body>,
    hashmap: KnnMapRead,
) -> Box<Future<Item = Response<Body>, Error = Error> + Send> {
    let body = req.into_body();
    let s = body
        .concat2()
        .into_future()
        .map_err(Error::from)
        .and_then(|buf| {
            debug!("Deserializing message");
            serialize_packed::read_message(
                &mut buf.as_ref(),
                ::capnp::message::ReaderOptions::default(),
            )
            .map_err(Error::from)
        })
        .and_then(move |message_reader| {
            debug!("Sending to Knn service");
            let request = fry!(message_reader.get_root::<knn_request_by_id::Reader>());
            let name = fry!(request.get_index_name());
            let index = fry!(Knn::get_index2(hashmap.clone(), name));
            debug!("Searching in index: {}", name);
            Either::A(Knn::search_id(index, request))
        })
        .and_then(move |builder| {
            debug!("Builing Response");
            let mut buffer = Vec::with_capacity(256);
            serialize_packed::write_message(&mut buffer, &builder)?;
            Ok(Response::new(Body::from(buffer)))
        });

    Box::new(s)
}

pub struct KnnService {
    pub state: Knn,
}

impl KnnService {
    pub fn new(state: Knn) -> KnnService {
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
        debug!("Receiving request");
        match (req.method(), req.uri().path()) {
            (&Method::POST, "/search") => {
                let hashmap = self.state.index_read.clone();
                let res = search(req, hashmap).map_err(|r| {
                    warn!("{:?}", r);
                    r
                });
                Box::new(res)
            }
            (&Method::POST, "/search2") => {
                let hashmap = self.state.index_read.clone();
                let res = search2(req, hashmap).map_err(|r| {
                    warn!("{:?}", r);
                    r
                });
                Box::new(res)
            }
            (&Method::POST, "/load") => {
                let write_handle = self.state.index_write.clone();
                let f = req
                    .into_body()
                    .concat2()
                    .map_err(|r| {
                        warn!("{:?}", r);
                        r
                    })
                    .map_err(Error::from)
                    .and_then(
                        move |buf| match serde_json::from_slice::<LoadRequest>(&buf) {
                            Err(e) => Err(Error::from(e)),
                            Ok(loadr) => {
                                match Knn::load(write_handle, &loadr.index_name, loadr.path) {
                                    Err(e) => Err(e),
                                    Ok(_) => Response::builder()
                                        .status(StatusCode::OK)
                                        .body(Body::empty())
                                        .map_err(Error::from),
                                }
                            }
                        },
                    )
                    .map_err(|r| {
                        warn!("{:?}", r);
                        r
                    });
                Box::new(f)
            }
            (&Method::GET, "/health") => Box::new(future::ok(
                Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from("OK"))
                    .unwrap(),
            )),
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
