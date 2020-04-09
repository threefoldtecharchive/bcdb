from bcdb_pb2_grpc import BCDBStub
from bcdb_pb2 import SetRequest, GetRequest, GetResponse

if __name__ == "__main__":
    import grpc
    import datetime

    count = 50000
    channel = grpc.insecure_channel("[::1]:50051")
    stub = BCDBStub(channel)

    request = SetRequest()

    start = datetime.datetime.now()
    for _ in range(count):
        _ = stub.Set(request)

    time_taken = datetime.datetime.now() - start
    print("50_000 empty requests in {}.{} s".format(time_taken.seconds, time_taken.microseconds))
    # or request = GetRequest()
    # request.id = 'my id'

    request.data = b"\0" * 1024
    start = datetime.datetime.now()
    for _ in range(count):
        _ = stub.Set(request)

    time_taken = datetime.datetime.now() - start
    print("50_000 1KB set requests in {}.{} s".format(time_taken.seconds, time_taken.microseconds))
