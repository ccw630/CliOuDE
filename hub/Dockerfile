FROM golang:1.18-alpine

ADD . /src
WORKDIR /src

RUN go build -v -o bin/clioude-hub

FROM alpine
COPY --from=0 /src/bin/clioude-hub /usr/bin/clioude-hub

ENTRYPOINT /usr/bin/clioude-hub