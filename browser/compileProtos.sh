#!/bin/bash
mkdir -p ./compiled_proto

for proto_file in ../protos/*.proto; do
  ./node_modules/.bin/grpc_tools_node_protoc \
    --plugin=protoc-gen-ts_proto=./node_modules/.bin/protoc-gen-ts_proto \
    --ts_proto_out=./compiled_proto \
    --ts_proto_opt=outputServices=nice-grpc,outputServices=generic-definitions,useExactTypes=false \
    --proto_path=../protos \
    "$proto_file"
done