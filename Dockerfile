FROM rust:1.57.0-slim-buster AS dist

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
            libssl-dev pkg-config

COPY ./ ./stream-rust-server

WORKDIR /stream-rust-server

RUN cargo build --release

RUN mkdir /out && cp /stream-rust-server/target/release/stream-rust-server /out/stream-rust-server

FROM debian:stable-20210902-slim AS runtime

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
            libssl-dev

RUN apt update && apt install netcat curl net-tools -y

COPY --from=dist /out/ /

EXPOSE 4222 5432

ENTRYPOINT ["/stream-rust-server"]