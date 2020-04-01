
## distributed storage

- all data is in ZDB DB files
    - the backend of ZDB
- once a ZDB DB file > max size KB they get stored on backend using ZeroSTOR
- in one system namespace we keep track of which backend stored object corresponds to ZDB backend DB files
- the max size per ZDB DB file is defined by $3bot user (cannot be smaller than 128KB)

## changes required to ZDB

- variable size for ZDB backend files
- when ZDB backend file is not there ask BCDB Server to retrieve missing BCDB DB File

## BCDB signing

- BCDB Server will hash each ZDB DB file before sending to zerostor
- the file send to zerostor will remember the hash and id of previous ZDB DB files for that namespace (keep e.g. 100 of them)
- which means everyone can verify per namespace that all ZDB DB files are consistent and have not been tampered with

### index storage

every 1 day the index per namespace gets compacted (see how best) and stored using zerostor.
In the system namespace we keep information as well to this index.

## reliability

each time we store a ZDB container or index compressed into ZeroSTOR
we append the full metadata of the system namespace in ZDB (metadata for the ZDB DB file  & index dumps)

## BCDB index can be lost

- anyone can starting from ZDB Files on ZDB re-create the BCDB Server index

## how can someone rebuild the BCDB as like any blockchain

- start from last metadata blob (as stored in last ZDB file on zerostor)
- for each namespace we want to retrieve
    - retrieve all ZDB backend files until we find the last index dump
    - restore the index with index dump
    - this means we have now BCDB active with all relevant ZDB DB files & index, when cache miss, the ZDB missing file will be retrieved.
    - for all ZDB DB files after the index dump, add the records to the index
    - while we get ZDB DB files back we check the hash chain for verification
    
