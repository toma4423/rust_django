# Rustのビルドには時間がかかるため、ビルドステージと実行ステージを分けます。
FROM rust:1.85-slim AS builder

# OpenSSLとPostgreSQLの開発用ライブラリが必要な場合があります
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# 開発用なのでビルド速度を優先。本番用なら --release を推奨。
RUN cargo build

# 実行環境
FROM debian:bookworm-slim
# 実行に必要なランタイムライブラリのみをインストール
RUN apt-get update && apt-get install -y libpq5 curl ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# バイナリ、テンプレート、静的ファイルをコピー
COPY --from=builder /app/target/debug/rust-django-starter /app/server
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/static /app/static

# Rocketの設定 (すべてのネットワークインターフェースで待機)
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

EXPOSE 8000
CMD ["/app/server"]
