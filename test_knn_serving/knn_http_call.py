import requests
from typing import *
import capnp

capnp.remove_import_hook()
knn_capnp = capnp.load("knn_serving_api/src/service.capnp")
http_url = "http://{}/".format("localhost:8080")

def load():
    load = requests.post(
        http_url + "load", json={"index_name": "toto2", "path": "dist/indexes/14018"}
    )
    print(load)

def search(v):
    request = knn_capnp.KnnRequest.new_message(
        indexName="toto2", algorithm="annoy", resultCount=10, searchK=10
    )
    request.init("vector", 101)
    b = request.to_bytes_packed()
    r = requests.post(http_url + "search", data=b)
    print(r)
    res = knn_capnp.KnnResponse.from_bytes_packed(r.content)
    return res


def search2(pid):
    request = knn_capnp.KnnRequestById.new_message(
        indexName="toto2", algorithm="annoy", resultCount=10, searchK=10, productId=pid
    )
    b = request.to_bytes_packed()
    r = requests.post(http_url + "search2", data=b)
    res = pid = knn_capnp.KnnResponse.from_bytes_packed(r.content)
    return res

res = search2(5931691376523631857)
ids = [pid.id for pid in res.items]
print(ids)

