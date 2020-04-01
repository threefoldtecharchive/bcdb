

- all data is in ZDB files
- once new ZDB files get above e.g. 128KB they get stored on backend using ZeroSTOR
- is combination of ZDB backend files (multiple namespaces)
- in one system namespace we keep track of which backend stored object corresponds to X ZDB backend DB files

## changes required to ZDB

- allow forced jump to 
