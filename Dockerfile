FROM rust:slim-bookworm AS builder
WORKDIR /usr/src/k4status
ENV TARGET x86_64-unknown-linux-musl
RUN apt-get update &&\
    apt-get --no-install-recommends -y install musl-tools &&\
    rustup target add "$TARGET"
COPY --exclude=target/ . .
RUN cargo install --target "$TARGET" --path .

FROM scratch
COPY --from=builder /usr/local/cargo/bin/k4status /
COPY config.json template.json /
COPY assets/ /assets
EXPOSE 3000
CMD ["./k4status"]