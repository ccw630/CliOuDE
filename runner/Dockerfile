FROM rust:1.59-buster

ADD . /src
WORKDIR /src

RUN cargo build --release

FROM ccw630/clioude-kernel
COPY --from=0 /src/target/release/clioude-runner /usr/bin/clioude-runner

ENTRYPOINT /usr/bin/clioude-runner