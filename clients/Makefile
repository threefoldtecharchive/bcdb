include = ../proto
proto = bcdb.proto

.PHONY: python go

all: python go js

python-deps:
	pip install --user grpcio-tools
	mkdir -p python/bcdb

python: python-deps
	python -m grpc_tools.protoc -I$(include) --python_out=python/bcdb --grpc_python_out=python/bcdb $(proto)

go-deps:
	mkdir -p go/bcdb/bcdb_proto

go: go-deps
	protoc -I$(include) $(proto) --go_out=plugins=grpc:go/bcdb/bcdb_proto

js-deps:
	mkdir -p js/bcdb

js: js-deps
	protoc -I$(include) $(proto) --js_out=import_style=commonjs:js/bcdb
	protoc -I$(include) $(proto) --grpc-web_out=import_style=commonjs,mode=grpcwebtext:js/bcdb
