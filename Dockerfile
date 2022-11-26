FROM rust as builder
RUN rustup toolchain add nightly
RUN rustup default nightly
RUN cargo +nightly install -f cargo-fuzz

ADD . /css-inline
WORKDIR /css-inline/css-inline/fuzz

RUN cargo fuzz build inline

# Package Stage
FROM ubuntu:20.04

COPY --from=builder /css-inline/css-inline/fuzz/target/x86_64-unknown-linux-gnu/release/inline /
