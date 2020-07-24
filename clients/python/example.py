from bcdb import Identity, Client, HTTPClient


def grpc_client_example():
    identity = Identity.from_seed("user.seed")
    client = Client("127.0.0.1:50051", identity, ssl=False)

    example = client.collection("random")

    key = example.set(
        b'hello world', {"example": "value", "tag2": "v2", ":invalid": "xxx"})
    obj = example.get(key)
    print("get: ", obj)

    obj = client.fetch(key)
    print("fetch: ", obj)

    example.update(key, b'updated string', {"example": "updated"}, acl=10)
    obj = example.get(key)
    print(obj)

    for id in example.list(example="updated"):
        print(id)

    for o in example.find(example="updated"):
        print(o)

    example.delete(key)
    obj = example.get(key)
    print(obj)


def rest_client_example():
    client = HTTPClient("/tmp/bcdb.sock")

    example = client.collection("random")

    key = example.set(
        b'hello world', {"example": "value", "tag2": "v2"})
    obj = example.get(key)
    print("get: ", obj)

    obj = client.fetch(key)
    print("fetch: ", obj)

    example.update(key, b'updated string', {"example": "updated"}, acl=10)
    obj = example.get(key)
    print(obj)

    for id in example.list(example="updated"):
        print(id)

    for o in example.find(example="updated"):
        print(o)

    # example.delete(key)
    # obj = example.get(key)
    # print(obj)


if __name__ == '__main__':
    # grpc_client_example()
    rest_client_example()
