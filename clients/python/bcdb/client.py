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
        """
        lists all acl objects
        """
        return self.__stub.List(types.ACLListRequest())

    def get(self, key: int):
        """
        gets an acl object given key
        """
        request = types.ACLGetRequest(key=key)
        return self.__stub.Get(request).acl

    def set(self, key: int, perm: str):
        """
        update an acl permissions string
        """
        request = types.ACLSetRequest(
            key=key,
            perm=perm,
        )

        self.__stub.Set(request)

    def grant(self, key: int, users: list):
        """
        grant new users to the acl group
        """
        request = types.ACLUsersRequest(
            key=key,
            users=users,
        )

        self.__stub.Grant(request)

    def revoke(self, key: int, users: list):
        """
        removes users from an acl group
        """
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
    def __init__(self, channel, collection, threebot_id: int = None):
        self.__metadata = None if threebot_id is None else (
            ("x-threebot-id", str(threebot_id)),)

        self.__stub = BCDBStub(channel)
        self.__collection = collection

    @property
    def collection(self):
        return self.__collection

    def __tags_from_meta(self, metadata):
        tags = dict()
        for tag in metadata.tags:
            tags[tag.key] = tag.value

        return tags

    def get(self, id: int) -> Object:
        """
        gets an object given object id
        """
        request = types.GetRequest(
            collection=self.collection,
            id=id,
        )

        response = self.__stub.Get(request, metadata=self.__metadata)
        tags = self.__tags_from_meta(response.metadata)

        return Object(
            id=id,
            data=response.data,
            tags=tags
        )

    def set(self, data: bytes, tags: dict = None, acl: int = None):
        """
        set creates a new object given data and tags, and optional acl key.

        :param data: data to set
        :param tags: optional tags associated with the object. useful for find operations
        :param acl: optional acl key
        :returns: new object id
        """
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

        return self.__stub.Set(request, metadata=self.__metadata).id

    def update(self, id: int, data: bytes = None, tags: dict = None, acl: int = None):
        """
        Update object given object id.

        :param data: optional update object data
        :param tags: optional update tags. new tags will override older tag values that has
                     the same tag name, new tags will be appended.
        :param acl: optional override object acl. note that only owner can set this field
                    even if the caller has a write permission on the object
        """
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

        self.__stub.Update(request, metadata=self.__metadata)

    def delete(self, id: int):
        """
        Mark the object as deleted
        """
        request = types.DeleteRequest(
            id=id,
            collection=self.collection,
        )

        self.__stub.Delete(request, metadata=self.__metadata)

    def list(self, **matches):
        """
        List all object ids that matches given tags
        """
        tags = list()
        for k, v in matches.items():
            tags.append(
                types.Tag(key=k, value=v)
            )

        request = types.QueryRequest(
            collection=self.collection,
            tags=tags,
        )

        for result in self.__stub.List(request, metadata=self.__metadata):
            yield result.id

    def find(self, **matches):
        """
        Find all objects that matches given tags

        Note: returned objects from fiend does not include data. so object.data will always be None
              to get the object data you will have to do a separate call to .get(id)
        """
        tags = list()
        for k, v in matches.items():
            tags.append(
                types.Tag(key=k, value=v)
            )

        request = types.QueryRequest(
            collection=self.collection,
            tags=tags,
        )

        for result in self.__stub.Find(request, metadata=self.__metadata):
            yield Object(
                id=result.id,
                data=None,
                tags=self.__tags_from_meta(result.metadata),
            )


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

    def collection(self, collection: str, threebot_id: int = None) -> BcdbClient:
        """
        Return a bcdb client

        :threebot_id: which threebot id instance to use, if None, use the
                      one directly connected to by this client
        """
        return BcdbClient(self.__channel, collection, threebot_id)
