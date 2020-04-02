include = ../proto
proto = bcdb.proto

.PHONY: python go

all: python go

python-deps:
	pip install --user grpcio-tools
	mkdir -p python/bcdb

python: python-deps
	python -m grpc_tools.protoc -I$(include) --python_out=python/bcdb --grpc_python_out=python/bcdb $(proto)

go-deps:
	mkdir -p go/bcdb

go: go-deps
	protoc -I$(include) $(proto) --go_out=plugins=grpc:go/bcdb
