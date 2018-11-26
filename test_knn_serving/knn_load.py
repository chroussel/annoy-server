import capnp

capnp.remove_import_hook()
knn_capnp = capnp.load('../knn_serving/src/service.capnp')
client = capnp.TwoPartyClient("localhost:8080")
service = client.bootstrap().cast_as(knn_capnp.KnnService)

load_request = service.load_request()
load_request.indexName = "toto"
load_request.indexPath = "/Users/c.roussel/sources/annoy-server/indexes/868"
load_promise = load_request.send()
res = load_promise.wait()
print(res)
