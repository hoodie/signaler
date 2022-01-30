alias b := build
alias r := run 

set dotenv-load := false

yarn := "yarn --no-color --emoji false --no-progress"

default:
  just --list

# server2 directory
build-server: build-webapp
  cd server2 && cargo build

# server2 directory
run-server:
  cd server2 && cargo run

# client lib
build-client:
  {{yarn}} --cwd client-lib build

# react webapp
@build-webapp: build-client
  {{yarn}} --cwd webapp webpack

# svelte webapp
build_svelte: build-client
  {{yarn}} --cwd webapp-svelte build

install:
  {{yarn}}

build: install build-webapp build-server
run: build-webapp run-server
