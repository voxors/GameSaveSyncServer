FROM rust:latest AS builder
WORKDIR /usr/src/app

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    libsqlite3-dev \
    npm && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app/frontend
COPY frontend/package.json package.json
COPY frontend/package-lock.json package-lock.json
COPY frontend/tsconfig.json tsconfig.json
COPY frontend/css css
COPY frontend/dist/static dist/static
COPY frontend/html html
COPY frontend/ts ts
RUN npm install
WORKDIR /usr/src/app

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY askama.toml askama.toml
COPY build.rs build.rs
COPY src src
COPY migrations migrations
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
COPY --from=builder --chown=appuser:appuser /usr/src/app/frontend/dist/generated /app/GameSaveServer/frontend/dist/generated
COPY --from=builder --chown=appuser:appuser /usr/src/app/frontend/dist/static /app/GameSaveServer/frontend/dist/static
WORKDIR /app/GameSaveServer
RUN mkdir -p data && chown appuser:appuser data
USER appuser
EXPOSE 3000
CMD ["./GameSaveServer"]
