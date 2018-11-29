import capnp

capnp.remove_import_hook()
knn_capnp = capnp.load('../knn_serving_api/src/service.capnp')
client = capnp.TwoPartyClient("localhost:8080")
service = client.bootstrap().cast_as(knn_capnp.KnnService)

request = knn_capnp.KnnRequest.new_message(
    indexName="toto", algorithm='annoy', resultCount=10, searchK=10)
request.init('vector', 100)

search = service.search_request()
search.request = request
request_promise = search.send()
res = request_promise.wait()
print(res)
