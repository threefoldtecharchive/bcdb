from bcdb_pb2_grpc import BCDBStub
from bcdb_pb2 import GetRequest, GetResponse, SetRequest, SetResponse, Tag

if __name__ == "__main__":
    import grpc
    from auth import auth_plugin_from_words

    user_id = 127
    with open("mywords.seed") as f:
        words = f.read().strip()

    auth_plugin = auth_plugin_from_words(user_id, words)
    call_cred = grpc.metadata_call_credentials(auth_plugin, name="auth gateway")

    # TODO: make sure we must need a secure_channel to use this call_cred
    # for now, we use a secure channel with local channel credentials
    channel_cred = grpc.local_channel_credentials()
    creds = grpc.composite_channel_credentials(channel_cred, call_cred)
    channel = grpc.secure_channel("127.0.0.1:50051", creds)

    # channel = grpc.insecure_channel("127.0.0.1:50051")
    stub = BCDBStub(channel)

    request = SetRequest(
        data=b"config here",
        metadata={"collection": "configs", "tags": [Tag(key="jumpscale.clients.redis", value="{}")]},
    )

    resp = stub.Set(request)
    print(f"id: {resp.id}")

    resp = stub.Get(GetRequest(collection="configs"))
    print(resp.data, resp.metadata)
