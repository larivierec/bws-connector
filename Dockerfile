FROM rust:1.91-alpine as builder
WORKDIR /usr/src/bws-connector

# Install build deps on Alpine
RUN apk add --no-cache build-base openssl-dev pkgconfig musl-dev ca-certificates

COPY . .
RUN cargo build --release

FROM alpine:3
RUN apk add --no-cache ca-certificates
WORKDIR /app

COPY --from=builder /usr/src/bws-connector/target/release/bws-connector /app/bws-connector
RUN chmod +x /app/bws-connector

ENTRYPOINT ["/app/bws-connector"]
