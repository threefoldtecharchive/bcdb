from bcdb import Identity, Client, HTTPClient


def grpc_client_example():
    identity = Identity.from_seed("user.seed")
    client = Client("127.0.0.1:50051", identity, ssl=False)

    example = client.collection("example")

    key = example.set(b'hello world', {"example": "value", "tag2": "v2"})
    obj = example.get(key)
    print(obj)

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
    identity = Identity.from_seed("user.seed")
    client = HTTPClient("http://127.0.0.1:50061", identity)

    example = client.collection("example")

    key = example.set(b'hello world', {"example": "value", "tag2": "v2"})
    obj = example.get(key)
    print(obj.headers)
    print(obj.text)

    example.update(key, b'updated string', {"example": "updated"}, acl=10)
    obj = example.get(key)
    print(obj.headers)
    print(obj.text)

    for o in example.find(example="updated"):
        print(o)

    example.delete(key)
    obj = example.get(key)
    print(obj)
    print(obj.headers)
    print(obj.text)


if __name__ == '__main__':
    rest_client_example()
