alias b := build
alias r := run
alias g := generate-protocol

set dotenv-load := false

deno := "deno"

default:
  just --list

# Generate TypeScript definitions from Rust protocol types
generate-protocol:
  cd protocol && cargo build --bin export_typescript --quiet && ../target/debug/export_typescript > generated.ts

# Alias for generate-protocol (same thing)
build-protocol: generate-protocol

# client lib (requires protocol to be generated first)
build-client: build-protocol
  {{deno}} task --cwd client-lib build

# react webapp
@build-webapp: build-client
  {{deno}} task --cwd webapp build

# svelte webapp
build_svelte: build-client
  {{deno}} task --cwd webapp-svelte build

# server directory
build-server: build-webapp
  cd server && cargo build

# server directory
run-server:
  cd server && cargo run

install:
  {{deno}} install

build: install build-protocol build-client build-webapp build-server
run: build run-server
