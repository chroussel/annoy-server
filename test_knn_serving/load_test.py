from locust import Locust, TaskSet, task, events
import capnp
import time

capnp.remove_import_hook()
knn_capnp = capnp.load('../knn_serving/src/service.capnp')


def fire_event(start_time, name, r):
    total_time = int((time.time() - start_time) * 1000)
    events.request_success.fire(
        request_type="search", name=name, response_time=total_time, response_length=0)


def search(service):
    request = knn_capnp.KnnRequest.new_message(
        indexName="toto", algorithm='annoy', resultCount=10, searchK=10)
    request.init('vector', 100)

    search = service.search_request()
    search.request = request
    request_promise = search.send()
    return request_promise


class CapnpRPCClient():
    def __init__(self, service):
        self.service = service

    def __getattr__(self, name):

        def wrapper(*args, **kwargs):
            start_time = time.time()
            try:
                result = search(self.service).then(
                    lambda l: fire_event(start_time, name, l)).wait()
            except:
                events.request_failed.fire(
                    request_type="search", name=name, response_time=total_time, response_length=0)

        return wrapper


class CapnpLocust(Locust):
    def __init__(self, *args, **kwargs):
        super(CapnpLocust, self).__init__(*args, **kwargs)

        proxy = capnp.TwoPartyClient("localhost:8080")
        client = proxy.bootstrap().cast_as(knn_capnp.KnnService)
        self.client = CapnpRPCClient(client)


class SearchApi(CapnpLocust):
    host = "http://127.0.0.1:8080/"
    min_wait = 1
    max_wait = 10

    class task_set(TaskSet):
        @task(10)
        def search(self):
            self.client.search()
