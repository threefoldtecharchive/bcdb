import base64
import time

import grpc

from collections import namedtuple

from mnemonic import Mnemonic
from nacl.signing import SigningKey


AuthOption = namedtuple("AuthOptions", ("id", "privkey", "expires"))
EnMnemonic = Mnemonic("english")


def get_auth_header(options):
    created = int(time.time())
    expires = created + options.expires

    key = SigningKey(options.privkey)

    headers = f"(created): {created}\n"
    headers += f"(expires): {expires}\n"
    headers += f"(key-id): {options.id}"

    signed_headers = key.sign(headers.encode())
    signature = base64.standard_b64encode(signed_headers.signature).decode()

    auth_header = f'Signature keyId="{options.id}",algorithm="hs2019",created="{created}",expires="{expires}",headers="(created) (expires) (key-id) ", signature="{signature}"'
    return "authorization", auth_header


class AuthGateway(grpc.AuthMetadataPlugin):
    def __init__(self, auth_options, *args, **kwargs):
        self.header = get_auth_header(auth_options)
        super().__init__(*args, **kwargs)

    def __call__(self, context, callback):
        """Implements authentication by passing metadata to a callback.
        Implementations of this method must not block.
        Args:
        context: An AuthMetadataContext providing information on the RPC that
            the plugin is being called to authenticate.
        callback: An AuthMetadataPluginCallback to be invoked either
            synchronously or asynchronously.
        """
        callback((self.header,), None)


def auth_plugin_from_words(user_id, words, expires=3600):
    key = EnMnemonic.to_entropy(words)
    options = AuthOption(user_id, bytes(key), expires)
    return AuthGateway(options)
