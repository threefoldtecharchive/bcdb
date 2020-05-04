# Peer To Peer (p2p) specifications
## Introduction

BCDBs should be able to communicate with each other to share and validate data. It means a BCDB server need to impersonate its user and be able to use his identity. A bcdb server then can create requests to other BCDB instance that are signed correctly. The receiving BCDB instance then can verify the identity
of the caller by verifying the user signature against his public key published on the explorer. Once identity of the caller is verified, the receiver bcdb
then can consult its local ACLs to see if the call is granted or not.

While most of this logic is already in place this raises other questions:
- How a BCDB instance find the public address of another instance. - Different ACLs that can be granted to users.

## Simple hybrid p2p architecture
- The explorer can work as a tracker, where bcdbs can retrieve public accessible addresses of other bcdb instances
  - Explorer high availability will become an issue.
- bcdb calls can have an optional 3bot id, this can either be local, or foreign 3bot id. In case of not provided, or local 3bot id the call is executed locally.
- if a foreign 3bot id is provided, the explorer is consulted for the public address of that 3bot, and then the call is done on behalf of the user to the remote 3bot.
- Before executing the call on the 3bot, we need to make sure we landed on the correct 3bot to avoid phishing. A foreign 3bot need to provide a proof of identity. this can be simply done by returning a signature of a random nonce, then the 3bot is trusted for a short period of time before another proof is required.
- Once the 3bot is trusted, his address is cached for sometime before another request to the explorer is required.
