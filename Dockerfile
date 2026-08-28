FROM node:22-alpine AS frontend
WORKDIR /build
COPY package.json package-lock.json ./
COPY frontend ./frontend
RUN npm ci && npm run build

FROM rust:1.98-bookworm AS backend
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 stockpromise \
    && useradd --uid 10001 --gid stockpromise --no-create-home stockpromise \
    && mkdir -p /app/dist /data \
    && chown -R stockpromise:stockpromise /data
WORKDIR /app
COPY --from=backend /build/target/release/stock-promise /app/stock-promise
COPY --from=frontend /build/dist /app/dist
ENV FRONTEND_DIR=/app/dist DATABASE_PATH=/data/stock-promise.db
EXPOSE 8080
USER 10001:10001
CMD ["/app/stock-promise"]

