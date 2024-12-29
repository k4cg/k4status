FROM rust:slim-bookworm as builder
WORKDIR /usr/src/k4status
ENV TARGET x86_64-unknown-linux-musl
RUN apt-get update &&\
    apt-get --no-install-recommends -y install musl-tools &&\
    rustup target add "$TARGET"
COPY . .
RUN cargo install --target "$TARGET" --path .

FROM scratch
COPY --from=builder /usr/local/cargo/bin/k4status /
EXPOSE 3000
CMD ["./k4status"]