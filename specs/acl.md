## Access control list
Each object is associated with a reference to a pre-configured ACL (access control list)
- The ACL holds a permission set + user set
  - permissions(read, write, delete, ...)
  - users(id, ...)
- The ACL api should provide the following functionality
  - create (acl) -> acl-ref
  - list() -> list ALL current available Acls
  - delete() -> not supported to avoid having objects that does not have a valid acl
  - update(acl-ref, add/remove user or change permissions)

### Internals
- ACLs should stored in a separate zdb namespace
