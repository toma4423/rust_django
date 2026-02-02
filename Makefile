.PHONY: run up down migrate test fmt lint build clean

# ===============================================
# Rust Django Starter Kit - Makefile
# django-admin 風のコマンドショートカット
# ===============================================

# サーバー起動 (Django: manage.py runserver)
run:
	DATABASE_URL=postgresql://user:password@localhost:5432/rust_django_db cargo run

# Docker Compose で起動
up:
	docker compose up

# Docker Compose でバックグラウンド起動
up-d:
	docker compose up -d

# Docker Compose 停止
down:
	docker compose down

# Docker Compose でビルドして起動
build-up:
	docker compose up --build

# テスト実行 (Django: manage.py test)
test:
	cargo test

# コードフォーマット
fmt:
	cargo fmt

# Lint チェック (clippy)
lint:
	cargo clippy -- -D warnings

# 型チェック (高速ビルドチェック)
check:
	cargo check

# 本番用ビルド
build:
	cargo build --release

# クリーンアップ
clean:
	cargo clean

# ヘルプ表示
help:
	@echo "Available commands:"
	@echo "  make run       - ローカルでサーバー起動"
	@echo "  make up        - Docker Compose で起動"
	@echo "  make up-d      - Docker Compose でバックグラウンド起動"
	@echo "  make down      - Docker Compose 停止"
	@echo "  make build-up  - Docker Compose でビルドして起動"
	@echo "  make test      - テスト実行"
	@echo "  make fmt       - コードフォーマット"
	@echo "  make lint      - Lint チェック"
	@echo "  make check     - 型チェック"
	@echo "  make build     - 本番用ビルド"
	@echo "  make clean     - クリーンアップ"
