include = ../proto
proto = bcdb.proto

.PHONY: python

all: python

python-deps:
	pip install --user grpcio-tools
	mkdir -p python/bcdb

python: python-deps
	python -m grpc_tools.protoc -I$(include) --python_out=python/bcdb --grpc_python_out=python/bcdb $(proto)
