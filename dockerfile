FROM rust:1.67 as builder
WORKDIR ~/rust/actix-web-blog
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/blog /usr/local/bin/blog
CMD ["blog"]
