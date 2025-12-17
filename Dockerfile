FROM rust:latest AS builder
WORKDIR /usr/src/app

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    libsqlite3-dev \
    npm && \
    rm -rf /var/lib/apt/lists/*

COPY package.json package.json
COPY package-lock.json package-lock.json
COPY tailwind.config.js tailwind.config.js

RUN npm install

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY build.rs build.rs
COPY src src
COPY templates templates
COPY migrations migrations
COPY static static
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libsqlite3-0 \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN groupadd -g 1000 appuser && \
    useradd -u 1000 -g appuser -s /bin/sh appuser

COPY --from=builder --chown=appuser:appuser /usr/src/app/migrations /app/GameSaveServer/migrations
COPY --from=builder --chown=appuser:appuser /usr/src/app/target/release/GameSaveServer /app/GameSaveServer
COPY --from=builder --chown=appuser:appuser /usr/src/app/generated /app/GameSaveServer/generated
COPY --from=builder --chown=appuser:appuser /usr/src/app/static /app/GameSaveServer/static
WORKDIR /app/GameSaveServer
RUN mkdir -p data && chown appuser:appuser data
USER appuser
VOLUME /app/GameSaveServer/data
EXPOSE 3000
CMD ["./GameSaveServer"]
