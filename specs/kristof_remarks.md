
## remarks v.1.0

- no schema support needed on server
- phonebook can be considered to be secure, phonebook will be secured by a blockchain of choice
- any object (binary, json, ...) can be stored on the BCDB server


## better definitions

- BCDB server for the server component
- BCDB client can be multiple ways
    - HTTP(S)?
    - grpc


## Questions

- why grpc? Personally don't see the benefit because we won't use schema's
- why not just redis like we already did or rest in first phase?
- if we would go for GRPC is it stateful, how link to web, when I looked in past was not ok

## todo

- need better specs for the tag layer and how it connects to sonic

## namespace id

- one ZDB namespace per BCDB namespace (is there limit)
- one sonic table or whatever they call it per BCDB namespace (is there limit?)
- BCDB namespace identification = $3botid_$packageid_$schemaid
    - $schemaid is unique per package id
    - $package id uniquer per per author who creates packages
    
## tags

- e.g ```color:red importance:10 customer:x location:europe.belgium.gent.korenlei```
- if spaces customername:'my customer'
- can search based on bcdb_namespace_id + x nr of tags
- question: can we suport prefixing of tags e.g. search on location:europe.* 

