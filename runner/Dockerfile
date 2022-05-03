FROM rust:1.59-alpine

ADD . /src
WORKDIR /src

RUN cargo build --release

FROM alpine
COPY --from=0 /src/runner/target/release/clioude-runner /usr/bin/clioude-runner

ENTRYPOINT /usr/bin/clioude-runner