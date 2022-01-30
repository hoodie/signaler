alias b := build
alias r := run 

set dotenv-load := false

yarn := "yarn --no-color --emoji false --no-progress"

default:
  just --list

# server directory
build-server: build-webapp
  cd server && cargo build

# server directory
run-server:
  cd server && cargo run

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
