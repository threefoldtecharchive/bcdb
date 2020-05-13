import json
from .auth import AuthGateway
from nacl.signing import SigningKey
from .generated import bcdb_pb2 as types
from .generated.bcdb_pb2_grpc import BCDBStub, AclStub
import base64
import grpc


class Client:
    def __init__(self, address, identity, ssl=True):
        auth_gateway = AuthGateway(identity, 3)
        call_cred = grpc.metadata_call_credentials(
            auth_gateway, name="bcdb-auth-gateway")

        channel_cred = None
        if ssl:
            channel_cred = grpc.ssl_channel_credentials()
        else:
            channel_cred = grpc.local_channel_credentials()

        credentials = grpc.composite_channel_credentials(
            channel_cred, call_cred)
        channel = grpc.secure_channel(address, credentials)

        self.__channel = channel
        self.__acl = AclClient(channel)

    @property
    def acl(self):
        return self.__acl


class AclClient:
    def __init__(self, channel):
        self.__stub = AclStub(channel)

    def list(self):
        return self.__stub.List(types.ACLListRequest())

    def get(self, key: int):
        request = types.ACLGetRequest(key=key)
        return self.__stub.Get(request).acl
