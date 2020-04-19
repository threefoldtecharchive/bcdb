# BCDB


## BCDB pre-requirements
BCDB uses zdb that is always running on local host ZDB must be running in `sequential` mode. Please check out [0-db](https://github.com/threefoldtech/0-db) for more information on how to install `zdb`.

```bash
zdb --mode seq
```

## Starting up BCDB
- Build bcdb
```bash
make
```
> Note that the code name for bcdb is `dbreboot`. This might stay for a while until the first official release. Hence the binary name is `dbreboot`
- check available options
```bash
# dbreboot --help
bcdb

USAGE:
    dbreboot [FLAGS] [OPTIONS]

FLAGS:
    -d, --debug      enable debug logging
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -l, --listen <listen>    listen on address [default: [::1]:50051]
    -m, --meta <meta>        directory where metadata is stored [default: $HOME/.bcdb-meta]
    -z, --zdb <zdb>          local zdb port [default: 9900]

```

## playing with bcdb
BCDB exposes a grpc service(s). We already have some clients generated (with some examples) please check `clients` directory.
In case there is no client generated in your preferred language, use the `proto/bcdb.proto` to generate one.

# Checklist
- [x] Set new object
- [x] Get object with ID
- [x] List objects that matches set of tags
- [x] Find objects that matches set of tags
  - find is similar to list, except `list` only returns object IDs, while `find` also return object full meta
- [x] Update object meta with ID
- Authentication
  - [ ] Specifications
- [-] ACL
  - [x] Assign ACL to object on Set
  - [x] Configure ACL (grant, revoke, and update permissions)
  - [ ] Check user access request against associated ACL
