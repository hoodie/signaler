FROM node:14.0.0-buster-slim as build_yarn
WORKDIR /usr/src/signaler
COPY static ./static
COPY client-lib ./client-lib
COPY webapp ./webapp
COPY webapp-svelte ./webapp-svelte
COPY package.json .
COPY yarn.lock .
RUN yarn 
RUN ls -l 
RUN yarn build:client
RUN yarn build:webapp
RUN yarn build:svelte

FROM rust:1.43-slim-buster as build_rust
WORKDIR /usr/src/signaler
ENV DOCKERIZE=1
COPY server ./server
COPY protocol ./protocol
COPY Cargo.toml . 
COPY Cargo.lock . 
RUN cargo build --release
 

FROM debian:buster-slim
RUN mkdir -p /opt/signaler/static
RUN mkdir -p /opt/signaler/server

COPY --from=build_yarn /usr/src/signaler/static                  /opt/signaler/static
COPY --from=build_yarn /usr/src/signaler/webapp-svelte           /opt/signaler/webapp-svelte
COPY --from=build_rust /usr/src/signaler/target/release/signaler /opt/signaler/server/

WORKDIR /opt/signaler/server
ENTRYPOINT /opt/signaler/server/signaler
