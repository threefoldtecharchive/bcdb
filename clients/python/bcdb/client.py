import json
from .auth import AuthGateway
from nacl.signing import SigningKey
from .generated import bcdb_pb2 as types
from .generated.bcdb_pb2_grpc import BCDBStub, AclStub
import base64
import grpc
from typing import NamedTuple


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


class Object(NamedTuple):
    id: int
    data: bytes
    tags: dict

    @property
    def acl(self):
        return int(self.tags[':acl']) if ':acl' in self.tags else None

    @property
    def size(self):
        return int(self.tags[':size']) if ':size' in self.tags else 0

    @property
    def created(self):
        return int(self.tags[':created']) if ':created' in self.tags else 0

    @property
    def updated(self):
        return int(self.tags[':updated']) if ':updated' in self.tags else 0


class BcdbClient:
    def __init__(self, channel, collection, threebot_id=None):
        self.__threebot_id = threebot_id
        self.__stub = BCDBStub(channel)
        self.__collection = collection

    @property
    def collection(self):
        return self.__collection

    def get(self, id: int) -> Object:
        request = types.GetRequest(
            collection=self.collection,
            id=id,
        )

        response = self.__stub.Get(request)
        tags = dict()
        for tag in response.metadata.tags:
            tags[tag.key] = tag.value

        return Object(
            id=id,
            data=response.data,
            tags=tags
        )

    def set(self, data: bytes, tags: dict = None, acl: int = None):
        _tags = list()
        for k, v in tags.items():
            _tags.append(
                types.Tag(key=k, value=v)
            )

        request = types.SetRequest(
            data=data,
            metadata=types.Metadata(
                collection=self.collection,
                acl=None if acl is None else types.AclRef(acl=acl),
                tags=_tags,
            )
        )

        return self.__stub.Set(request).id

    def update(self, id: int, data: bytes = None, tags: dict = None, acl: int = None):
        _tags = list()
        for k, v in tags.items():
            _tags.append(
                types.Tag(key=k, value=v)
            )

        request = types.UpdateRequest(
            id=id,
            data=None if data is None else types.UpdateRequest.UpdateData(
                data=data),
            metadata=types.Metadata(
                collection=self.collection,
                acl=None if acl is None else types.AclRef(acl=acl),
                tags=_tags,
            )
        )

        self.__stub.Update(request)


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

    def bcdb(self, collection: str, threebot_id: int = None) -> BcdbClient:
        """
        Return a bcdb client

        :threebot_id: which threebot id instance to use, if None, use the
                      one directly connected to by this client
        """
        return BcdbClient(self.__channel, collection, threebot_id)
