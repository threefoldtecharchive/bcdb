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

        # channel_cred = None
        # if ssl:
        #     channel_cred = grpc.ssl_channel_credentials()
        # else:
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

    def create(self, perm: str = 'r--', users: list = None):
        """
        :param perm: string in format `rwd`, set the missing permission to `-`
        """
        request = types.ACLCreateRequest(
            acl=types.ACL(
                perm=perm,
                users=users
            )
        )
        return self.__stub.Create(request).key

    def list(self):
        return self.__stub.List(types.ACLListRequest())

    def get(self, key: int):
        request = types.ACLGetRequest(key=key)
        return self.__stub.Get(request).acl

    def set(self, key: int, perm: str):
        request = types.ACLSetRequest(
            key=key,
            perm=perm,
        )

        self.__stub.Set(request)

    def grant(self, key: int, users: list):
        request = types.ACLUsersRequest(
            key=key,
            users=users,
        )

        self.__stub.Grant(request)

    def revoke(self, key: int, users: list):
        request = types.ACLUsersRequest(
            key=key,
            users=users,
        )

        self.__stub.Revoke(request)
