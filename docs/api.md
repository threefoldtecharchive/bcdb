## DBREBOOT API

## Acl endpoints

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

Return created acl id

### GET `/acl/:key`

Returns an acl by key

### PUT `/acl/:key`

Modifies an acl by key

Put example with json:
```json
{
    "perm": "rwd"
}
```

### POST `/acl/:key/grant`

grants acl permissions to users

Post example with json:
```json
{
    "users": [54, 68]
}
```

### POST `/acl/:key/revoke`

revokes acl permissions from users

Post example with json:
```json
{
    "users": [54, 68]
}
```