@echo off
REM Generate Python gRPC code from .proto files (Windows)

set PROTO_DIR=..\Rust\protos
set PYTHON_OUT_DIR=.\generated

if not exist %PYTHON_OUT_DIR% mkdir %PYTHON_OUT_DIR%

python -m grpc_tools.protoc ^
    -I%PROTO_DIR% ^
    --python_out=%PYTHON_OUT_DIR% ^
    --grpc_python_out=%PYTHON_OUT_DIR% ^
    %PROTO_DIR%\database.proto ^
    %PROTO_DIR%\ml_inference.proto

if %ERRORLEVEL% EQU 0 (
    echo Python gRPC code generated successfully in %PYTHON_OUT_DIR%
) else (
    echo Failed to generate Python gRPC code
    exit /b 1
)

