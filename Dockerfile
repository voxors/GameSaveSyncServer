FROM rust:latest AS builder
WORKDIR /usr/src/app

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo fetch
RUN cargo build --release


FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libsqlite3-0 \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/GameSaveServer ./GameSaveServer
RUN mkdir -p data
VOLUME /app/data
EXPOSE 3000
CMD ["./GameSaveServer"]
