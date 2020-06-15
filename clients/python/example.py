from bcdb import Identity, Client, HTTPClient

if __name__ == "__main__":
    identity = Identity.from_seed("user.seed")
    client = HTTPClient("http://localhost:50061", identity)
    # collection = client.collection("http")

    # id = collection.set(
    #     "Some data goes here", {"name": "test", "parent": "some parent with\nnewlines"}, acl='10')
    # response = collection.get(id)

    # print(response)
    # print(response.headers)
    # print(response.text)

    # print("update", id)
    # response = collection.update(id,
    #                              "Updated data", {"name": "test", "parent": "new value"})

    # print("Update Response:", response)

    # response = collection.get(id)

    # print(response)
    # print(response.headers)
    # print(response.text)

    acl = client.acl

    key = acl.create("r--", [1, 2])

    print("ACL ID", key)

    acl.set(key, 'r-d')
    response = acl.get(key)

    print("ACL:", response)

    acl.set(0, "rwd")

    print("grant", acl.grant(key, [2, 3]))

    print("revokie", acl.revoke(key, [1]))

    for acl in acl.list():
        print(acl)
