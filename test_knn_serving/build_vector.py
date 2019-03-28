import requests
from typing import *
import capnp

capnp.remove_import_hook()
knn_capnp = capnp.load("knn_serving_api/src/service.capnp")

request = knn_capnp.KnnRequest.new_message(
        indexName="all", algorithm='annoy', resultCount=10, searchK=10)
request.init('vector', 100)
b = request.to_bytes_packed()

with open("request.vector", "wb") as f:
    f.write(b)