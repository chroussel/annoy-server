import requests
import capnp


http_url = "URL"
capnp.remove_import_hook()
knn_capnp = capnp.load('../knn_serving_api/src/service.capnp')

load = requests.post(http_url + "/load", json={
    "index_name": "toto",
    "path": "dist/indexes/868"
})

print(load)

request = knn_capnp.KnnRequest.new_message(
    indexName="toto", algorithm='annoy', resultCount=10, searchK=10)
request.init('vector', 100)
b = request.to_bytes_packed()
r = requests.post(http_url + "/search", data=b)

res = knn_capnp.KnnResponse.from_bytes_packed(r.content)
print(res)
