set shell := ["powershell.exe", "-c"]

PROTOC := "protoc.exe"
PROTO_DIR := "proto"
OUT_RUST := "core/src/codegen/protos"
OUT_GO := "client-go/codegen/protos"

default:
    just --summary

gen-rust:
    {{PROTOC}} --proto_path={{PROTO_DIR}} --prost_out={{OUT_RUST}} {{PROTO_DIR}}/protocol.proto

gen-go:
    {{PROTOC}} --proto_path={{PROTO_DIR}} --go_out={{OUT_GO}} {{PROTO_DIR}}/protocol.proto

gen-all: gen-rust gen-go

build-debug PACKAGE:
    cargo build                         \
      --package {{ PACKAGE }}

build-release PACKAGE:
    cargo build                         \
      --release                         \
      --package {{ PACKAGE }}

build-tiny PACKAGE:
    Write-Host $env:RUSTFLAGS
    cargo +nightly build                            \
      -Z build-std=std,panic_abort                  \
      -Z build-std-features="optimize_for_size"     \
      --release                                     \
      --package {{ PACKAGE }}