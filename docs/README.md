# Index
- [Peer 2 Peer](#peer2peer)
- [Example usage](#example_usage)

## Peer 2 Peer
By default bcdb will proxy any call that need to be handled by a different instance to the right peer. Bcdb will alway check for the `x-threebot-id` metadata header in all BCDB grpc calls. And then will take the decision either to handle the request locally, or forward it to a different `bcdb` instance.

By default `bcdb` will lookup the proper peer endpoint using the explorer (as configured by the `--explorer` flag). This can be overridden (usually for development) by providing a `--peers-file` flag

The `peers-file` must be formatted as separate peers objects (not a json list) for example:
```json
{"id": 1, "host": "http://host-1:1234", "pubkey": "34c77fdf6c5ef24d5a6981be06f9109ba83b7e306cfad8141ce5f572b647cbeb"}
{"id": 2, "host": "http://host-2:1234", "pubkey": "8d0ba0d199426a71d5cb933406ab3296db5441384a5c5a39f4435130cfb688dc"}
```

Once this list is provided, bcdb will **ONLY** use it for peer ids look up.

Note the following:
- When bcdb detect that the request must be forwarded, no authentication is applied, and the request (and its authentication data) are forwarded as is. Hence the remote peer might still return `unauthorized` if you couldn't provide proper identity, or have access to that peer (according to its ACLs)
- Currently, peer validation is not implemented, hence if you are provided improper `peers-file` you might end up on a peer that is impersonating someone's else identity, hence might receive sensitive information from you. This will change in the future.

# Example Usage
Start 2 bcdb instances
```
./dbreboot -d --explorer 'https://explorer.devnet.grid.tf/explorer' \
    --listen 0.0.0.0:50051
    --seed-file=$HOME/user-1.seed \
    --peers-file peers.json
```

And on another terminal do
```
./dbreboot -d --explorer 'https://explorer.devnet.grid.tf/explorer' \
    --listen 0.0.0.0:50052
    --seed-file=$HOME/user-2.seed \
    --peers-file peers.json
```

You will end up with 2 instances running on ports `50051`, and `50052`.

## Client.
please check the `clients/go/example.go` client for a general usage of the client. There is a code comment that explain how to do a call to a peer by using `x-threebot-id` meta.
