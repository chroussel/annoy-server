use annoy_rs::idmapping::MappingIndex;
use service_capnp::knn_response;
use std::sync::Arc;

pub fn create_response_from_vectors(
    index: Arc<MappingIndex<i64>>,
    response_builder: knn_response::Builder,
    ids: &[i64],
    distances: &[f32],
) {
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
}

pub fn capnp_error_from_err(e: annoy_rs::err::Error) -> capnp::Error {
    capnp::Error::failed(format!("error in index: {:?}", e))
}
