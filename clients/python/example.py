from bcdb import Identity, Client, HTTPClient

if __name__ == "__main__":
    identity = Identity.from_seed("user.seed")
    client = HTTPClient("http://localhost:3030", identity)
    collection = client.collection("http")

    response = collection.set(
        "Some data goes here", {"name": "test", "parent": "some parent with\nnewlines"}, acl='10')

    print(response.text)

    id = response.text
    response = collection.get(id)

    print(response)
    print(response.headers)
    print(response.text)

    print("update", id)
    response = collection.update(id,
                                 "Updated data", {"name": "test", "parent": "new value"})

    print(response.text)

    response = collection.get(id)

    print(response)
    print(response.headers)
    print(response.text)
