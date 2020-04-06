from bcdb_pb2_grpc import BCDBStub
from bcdb_pb2 import GetRequest, GetResponse

if __name__ == '__main__':
    import grpc
    channel = grpc.insecure_channel("[::1]:50051")
    stub = BCDBStub(channel)

    request = GetRequest(id="my id")
    # or request = GetRequest()
    # request.id = 'my id'

    response = stub.Set(request)
