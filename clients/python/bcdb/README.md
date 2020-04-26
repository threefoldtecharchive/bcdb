# Python client

## Running server

You must have a user registered at explorer (main, testnet, or devnet).

If you use `jumpscale`, you can get your user ID and words as from `j.me`, just make sure you already did `j.me.configure()`


```
JSX> j.me

## jumpscale.threebot.me
ID: 4
 - name                : default
 - tid                 : 122
 - tname               : myname.3bot
 - email               : mail@host.com
 ...
```

User id is `tid`, to get your words:

```
JSX> j.me.encryptor.words
'hamada ellol fol el fol...'
```

You can put it to a file and run bcdb server with it as:

```
/target/release/dbreboot --explorer https://explorer.grid.tf/explorer/ --seed-file mywords.seed -l 127.0.0.1:50051
```



## Setup

Install requirements

```
pip3 install -r requirements.txt
```

For now, if you used the same words file, you will get a full access over bcdb, just add `mywords.seed` to the same directory, also, modify this example.py file with your correct usr id instead of `user_id = 127`

```
python3 example.py
```


## Authentication

`auth.py` implements client authentication using a signed header with signature, and it sends it as `authorization` header for each call using a `AuthMetadataPlugin`, To know more about per-call and channel authentication, see https://github.com/grpc/grpc/tree/master/examples/python/auth.
