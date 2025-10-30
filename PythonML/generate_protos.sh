#!/bin/bash
# Generate Python code from proto files

mkdir -p generated

python -m grpc_tools.protoc \
    -I../Rust/protos \
    --python_out=./generated \
    --grpc_python_out=./generated \
    ../Rust/protos/ml_inference.proto

echo "✅ Proto files generated in generated/"

