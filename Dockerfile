# STAGE 1: Build Binary
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# STAGE 2: Runtime Image Mungil
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/qraa_architecture /app/qraa_architecture

EXPOSE 8080
ENV PORT=8080
ENV RUST_LOG=info

CMD ["/app/qraa_architecture"]