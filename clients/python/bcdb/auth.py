import base64
import time

import grpc

from typing import NamedTuple

from mnemonic import Mnemonic
from nacl.signing import SigningKey


class AuthOptions(NamedTuple):
    id: int
    key: SigningKey
    expires: int


EnMnemonic = Mnemonic("english")


# from https://github.com/grpc/grpc/tree/master/examples/python/auth
class AuthGateway(grpc.AuthMetadataPlugin):
    def __init__(self, auth_options, *args, **kwargs):
        self.auth_options = auth_options
        super().__init__(*args, **kwargs)

    def get_auth_header(self):
        created = int(time.time())
        expires = created + self.auth_options.expires

        headers = f"(created): {created}\n"
        headers += f"(expires): {expires}\n"
        headers += f"(key-id): {self.auth_options.id}"

        signed_headers = self.auth_options.key.sign(headers.encode())
        signature = base64.standard_b64encode(signed_headers.signature).decode()

        auth_header = f'Signature keyId="{self.auth_options.id}",algorithm="hs2019",created="{created}",expires="{expires}",headers="(created) (expires) (key-id)",signature="{signature}"'
        return "authorization", auth_header

    def __call__(self, context, callback):
        """Implements authentication by passing metadata to a callback.
        Implementations of this method must not block.
        Args:
        context: An AuthMetadataContext providing information on the RPC that
            the plugin is being called to authenticate.
        callback: An AuthMetadataPluginCallback to be invoked either
            synchronously or asynchronously.
        """
        callback((self.get_auth_header(),), None)


def auth_plugin_from_words(user_id, words, expires=3):
    entropy = EnMnemonic.to_entropy(words)
    signing_key = SigningKey(bytes(entropy))
    options = AuthOptions(user_id, signing_key, expires)
    return AuthGateway(options)
