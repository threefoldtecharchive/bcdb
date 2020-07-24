# BCDB Rest API

## Authorization
All request to BCDB must provide an `authorization` header. The `authorization` header is *[httpsig](https://tools.ietf.org/html/draft-cavage-http-signatures-12)* header

Please check the reference [python implementation](https://github.com/threefoldtech/bcdb/blob/bed9aa93b86c6d14f8534a6627bba1a4a75f2037/clients/python/bcdb/auth.py#L60) here.


## Data endpoints
Data endpoints are used to store, get, and find objects from bcdb.

### Peer 2 Peer
All requests to data endpoints accepts an optional header `x-threebot-id`. If the header is not set (or equals the instance ID), the bcdb instance handles the call locally. If the header is provided is not equal to the bcdb instance id, the BCDB instance will loop the explorer for the address of the bcdb instance of that user and forward the call to that instance (behind the scene).

### POST `/db/:collection`
Request body is your entire data. Currently we have a limitation of 4MB on the rest api as a protection mechanism until the large file support is implemented.

The POST request accepts the following headers:
- `x-acl: <acl-key>` sets the object [ACL](#acl-enpoints)
- `x-tags: <tags>` tags is a json serialized dict of tags (key/value)

Returns object id (json)

### GET `/db/:collection/:id`
Gets an object from the database. The object data is the entire response body.

The GET response also return the following headers:
- optional `x-acl: <acl-key>` the acl associated with this object (if
set)
- `x-tags: <tags>` the object tags as a dict in json format

### HEAD `/db/:collection/:id`
Gets an object metadata from the database.

The HEAD response returns the following headers:
- optional `x-acl: <acl-key>` the acl associated with this object (if
set)
- `x-tags: <tags>` the object tags as a dict in json format

### DELETE `/db/:collection/:id`
Marks object as deleted.

> Note after deletion, object remains accessible. it's only flagged with a special flag that it was deleted.

### PUT `/db/:collection/:id`
Updates an object. If a request body is provided, it overrides the object data. New tags (provided as `x-tags`) are appended to the object tags or override the old value if key already exists. Also override acl using the `x-acl` tag if provided.

### GET `/db/:collection`
The find interface to find object(s) using tags. It accepts an arbitrary query string based on the tags you used to store the object in the first place.

> **Note**: due to a bug in the server router, the query params must always be provided, to do an empty query (find everything) use `?_=` as query string. (for example `GET http:://localhost:50061/db/mycollection/?_=`)

#### Different find modes
You can select the `find` mode, using an optional header `x-find-mode`. This only supports 2 modes at the moment
- `find` this is the default mode if the header is not set.
Returns a stream of json object (not a list). The object ONLY contains the id and the metadata (tags) but not the content, if you need to retrieve the content a separate GET call must be done.
- `list` this has to be selected by setting the `x-find-mode: list` header. In this mode the returned objects are just the ids of your objects that are matching your query. No tags are returned

### DELETE `/db/:collection`
The delete interface to delete object(s) using tags. It accepts an arbitrary query string based on the tags you used to store the object in the first place.

> **Note**: due to a bug in the server router, the query params must always be provided, to do an empty query (delete everything) use `?_=` as query string. (for example `DELETE http:://localhost:50061/db/mycollection/?_=`)

> **Note**: due to some limitation in the index implementation. A patch delete operation is heavy, because a find operation need to be executed first, then collect all the matching keys, then iterate over all keys and delete them. So this operation is really not recommended right now, except for small result set.

## Acl endpoints
Acl are use to configure `access control list` groups. A single ACL object is a group of user ids, associated with a permission string. Then the same object can be assigned to multiple objects at the same time.

An object with no associated ACL will only be accessible by the BCDB owner.

### Permission String
Similar to linux filesystem permissions, a permission string is of length 3 formatted as `[r-][w-][d-]`.

Examples:
- `r--` means group only has read access
- `-w-` means group only has write access. (they can update the object but not read it)
- `--d` means group can delete object
- Any combination of 2 or 3 permissions is possible:
 - `rw-` read/write access.
 - `rwd` full access to the object

> Only owner of BCDB can manage the acl permissions with the API

### GET `/acl`

Lists all the acl's

### POST `/acl`

Creates an acl.

Post example with json:
```json
{
    "perm": "r--",
    "users": [54, 68]
}
```

Return created acl `id`

### GET `/acl/:key`

Returns an acl by key

### PUT `/acl/:key`

Modifies an acl permission string by acl key

Put example with json:
```json
{
    "perm": "rwd"
}
```

### POST `/acl/:key/grant`

Adds a user (or more) to the ACL.

Post example with json:
```json
{
    "users": [54, 68]
}
```

Returns number of newly added users.

### POST `/acl/:key/revoke`

Removes a user (or more) from the ACL.

Post example with json:
```json
{
    "users": [54, 68]
}

```
Returns number of removed users.
