use annoy_rs::idmapping::MappingIndex;
use err::Error;
use knn_serving_api::service_capnp::{knn_request, knn_response, knn_service};
use std::sync::Arc;

pub fn capnp_error_from_err(e: Error) -> capnp::Error {
    capnp::Error::failed(format!("error in index: {:?}", e))
}
