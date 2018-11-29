@0x96daa2e5618c8aff;

struct KnnRequest {
    indexName @0 :Text;
    algorithm @1 :Algorithm;
    resultCount @2 :Int32;
    searchK @3 :Int32;
    vector @4 :List(Float32);

    enum Algorithm {
        annoy @0;
    }
}

struct KnnResponse {
    resultCount @0 :Int32;
    items @1 :List(Item);

    struct Item {
        id @0 :Int64;
        vector @1 :List(Float32);
        distance @2 :Float32;
    }
}

interface KnnService {
    search @0 (request :KnnRequest) -> (response :KnnResponse);
    load @1(indexName :Text, indexPath :Text);
}