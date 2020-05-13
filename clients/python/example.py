from bcdb import Identity, Client

if __name__ == "__main__":
    identity = Identity.from_seed("/home/azmy/tmp/tfuser/dev.user.seed")
    client = Client("127.0.0.1:50051", identity, ssl=False)

    key = client.acl.create(users=[1, 2])
    for acl in client.acl.list():
        print("in list")
        print(acl)

    got = client.acl.get(key)
    print("got")
    print(got)

    client.acl.set(key, 'rw-')
    client.acl.grant(key, [5, 6])
    got = client.acl.get(key)
    print("after update")
    print(got)

    # import grpc
    # from .auth import auth_plugin_from_words

    # user_id = 127
    # with open("mywords.seed") as f:
    #     words = f.read().strip()

    # auth_plugin = auth_plugin_from_words(user_id, words)
    # call_cred = grpc.metadata_call_credentials(
    #     auth_plugin, name="auth gateway")

    # # TODO: make sure we must need a secure_channel to use this call_cred
    # # for now, we use a secure channel with local channel credentials
    # channel_cred = grpc.local_channel_credentials()
    # creds = grpc.composite_channel_credentials(channel_cred, call_cred)
    # channel = grpc.secure_channel("127.0.0.1:50051", creds)

    # # channel = grpc.insecure_channel("127.0.0.1:50051")
    # stub = BCDBStub(channel)

    # request = SetRequest(
    #     data=b"config here",
    #     metadata={"collection": "configs", "tags": [
    #         Tag(key="jumpscale.clients.redis", value="{}")]},
    # )

    # resp = stub.Set(request)
    # print(f"id: {resp.id}")

    # resp = stub.Get(GetRequest(collection="configs"))
    # print(resp.data, resp.metadata)
