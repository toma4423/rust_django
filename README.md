# Rust Django Starter Kit

<div align="center">

**DjangoユーザーのためのRust Webアプリケーション開発スターターキット**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Rocket](https://img.shields.io/badge/Rocket-0.5-red.svg)](https://rocket.rs/)
[![SeaORM](https://img.shields.io/badge/SeaORM-1.1-blue.svg)](https://www.sea-ql.org/SeaORM/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

</div>

---

## 📖 概要

Djangoの設計パターンをRustで再現したスターターキットです。Djangoユーザーが「これなら自分も書ける！」と思える構造と開発体験を提供します。

### ✨ 特徴

- 🔐 **セキュリティ重視**: Argon2パスワードハッシュ、CSRF対策実装済み
- 🎨 **Django Admin風UI**: 管理画面をDjangoスタイルで完全再現
- 📚 **教育的コード**: 全コードに「Djangoでいうとこれ」コメント付き
- ⚡ **HTMX同梱**: 複雑なJSフレームワーク不要でSPA風の挙動
- 🧪 **テスト済み**: ユニットテスト4件通過

---

## 🚀 クイックスタート

### 前提条件

以下がインストールされていることを確認してください：

| ツール | バージョン | 確認コマンド |
|--------|-----------|-------------|
| Rust | 1.70以上 | `rustc --version` |
| Cargo | 1.70以上 | `cargo --version` |
| PostgreSQL | 15以上 | `psql --version` |
| Docker (任意) | 20以上 | `docker --version` |

### 方法1: ローカル環境でのセットアップ

```bash
# 1. リポジトリをクローン
git clone <repository-url>
cd RustDjango

# 2. PostgreSQLデータベースを作成
createdb rust_django_db

# 3. 環境変数を設定
export DATABASE_URL=postgresql://your_user:your_password@localhost:5432/rust_django_db

# 4. 依存関係をインストールしてビルド
cargo build

# 5. サーバーを起動（マイグレーションは自動実行）
cargo run

# 6. ブラウザでアクセス
# http://localhost:8000
```

### 方法2: Docker Composeでのセットアップ

```bash
# 1. リポジトリをクローン
git clone <repository-url>
cd RustDjango

# 2. Docker Composeで起動
docker compose up --build

# 3. ブラウザでアクセス
# http://localhost:8000
```

### 方法3: Makefileを使用（推奨）

```bash
# 開発用コマンド一覧を表示
make help

# データベースを起動してサーバーを実行
make up      # PostgreSQLコンテナ起動
make run     # サーバー起動

# その他のコマンド
make test    # テスト実行
make fmt     # コードフォーマット
make lint    # Lintチェック
```

---

## 🔑 初回ログイン

アプリケーション起動後、以下の認証情報でログインできます：

| 項目 | 値 |
|------|-----|
| URL | http://localhost:8000 |
| ユーザー名 | `admin` |
| パスワード | `admin` |

> ⚠️ **本番環境では必ずパスワードを変更してください**

---

## 📂 プロジェクト構成

```
RustDjango/
├── 📄 Cargo.toml           # 依存関係定義 (≈ requirements.txt + settings.py)
├── 📄 Makefile             # 開発用コマンド (≈ manage.py)
├── 📄 compose.yaml         # Docker Compose設定
├── 📄 Dockerfile           # コンテナ定義
│
├── 📁 src/                 # Rustソースコード
│   ├── main.rs             # エントリーポイント (≈ urls.py)
│   ├── db.rs               # DB接続 (≈ settings.DATABASES)
│   ├── auth_utils.rs       # パスワードハッシュ (≈ django.contrib.auth.hashers)
│   ├── csrf.rs             # CSRF対策 (≈ CsrfViewMiddleware)
│   ├── errors.rs           # エラー型 (≈ django.http.Http404など)
│   │
│   ├── 📁 controllers/     # ビューロジック (≈ views.py)
│   │   ├── admin.rs        # 管理画面CRUD
│   │   └── auth.rs         # ログイン/ログアウト
│   │
│   ├── 📁 entities/        # モデル定義 (≈ models.py)
│   │   └── user.rs         # Userモデル
│   │
│   ├── 📁 guards/          # 認証ガード (≈ @login_required)
│   │   └── auth.rs         # AuthenticatedUser, AdminUser
│   │
│   └── 📁 services/        # ビジネスロジック (≈ managers.py)
│       └── user_service.rs # UserService
│
├── 📁 migration/           # マイグレーション (≈ migrations/)
│   └── src/
│       ├── lib.rs
│       ├── m20220101_000001_create_user_table.rs
│       └── m20220102_000001_create_admin_user.rs
│
├── 📁 templates/           # Teraテンプレート (≈ templates/)
│   ├── base.html.tera
│   ├── index.html.tera
│   └── 📁 admin/
│       ├── base.html.tera  # サイドバー付きベース
│       ├── list.html.tera  # ユーザー一覧
│       └── form.html.tera  # ユーザー作成/編集
│
└── 📁 static/              # 静的ファイル (≈ static/)
    └── css/
        └── style.css       # Django Admin風CSS
```

---

## 🛠 技術スタック

| コンポーネント | 使用技術 | Djangoでの相当 |
|---------------|---------|---------------|
| 言語 | Rust 1.70+ | Python |
| Webフレームワーク | Rocket 0.5 | Django |
| ORM | SeaORM 1.1 | Django ORM |
| テンプレート | Tera | Django Template |
| 認証 | Argon2 + Cookie | django.contrib.auth |
| CSRF | 独自実装 | CsrfViewMiddleware |
| フロントエンド | HTMX + Bootstrap 5 | HTMX / Vanilla JS |

---

## 📋 開発ガイド

### 新しいモデルを追加する

```bash
# 1. マイグレーションファイルを作成
cd migration
touch src/m20240101_000001_create_post_table.rs

# 2. entities/にモデル定義を追加
touch src/entities/post.rs

# 3. entities/mod.rsに登録
echo 'pub mod post;' >> src/entities/mod.rs
```

### 新しいコントローラを追加する

```rust
// src/controllers/post.rs
use rocket_dyn_templates::{Template, context};
use crate::guards::auth::AuthenticatedUser;

#[get("/posts")]
pub fn list_posts(_user: AuthenticatedUser) -> Template {
    Template::render("posts/list", context! {})
}
```

### 認証が必要なエンドポイント

```rust
// AuthenticatedUser を引数に追加するだけ（@login_required相当）
#[get("/protected")]
pub fn protected_route(user: AuthenticatedUser) -> String {
    format!("Hello, {}!", user.username)
}

// 管理者のみアクセス可能（@staff_member_required相当）
#[get("/admin-only")]
pub fn admin_route(_admin: AdminUser) -> String {
    "Welcome, Admin!".to_string()
}
```

---

## 🧪 テスト

```bash
# 全テストを実行
cargo test

# 特定のテストを実行
cargo test csrf

# 詳細出力
cargo test -- --nocapture
```

### テスト結果

```
running 4 tests
test auth_utils::tests::test_password_hash_and_verify ... ok
test auth_utils::tests::test_different_passwords_produce_different_hashes ... ok
test csrf::tests::test_csrf_token_generation ... ok
test csrf::tests::test_csrf_token_verification ... ok

test result: ok. 4 passed; 0 failed
```

---

## 🔐 セキュリティ機能

| 機能 | 実装状況 | 説明 |
|------|---------|------|
| パスワードハッシュ | ✅ | Argon2id使用 |
| CSRF対策 | ✅ | トークンベース（1時間有効） |
| Cookie署名 | ✅ | Rocket Private Cookie |
| XSS対策 | ✅ | Teraの自動エスケープ |

---

## 📦 Makefileコマンド一覧

| コマンド | 説明 | Djangoでの相当 |
|---------|------|---------------|
| `make run` | サーバー起動 | `python manage.py runserver` |
| `make test` | テスト実行 | `python manage.py test` |
| `make fmt` | コードフォーマット | `black .` |
| `make lint` | Lintチェック | `flake8` |
| `make up` | DB起動 | (Docker) |
| `make down` | DB停止 | (Docker) |
| `make build` | 本番ビルド | - |
| `make clean` | クリーンアップ | - |

---

## 🔄 Django → Rust 対応表

| Django | Rust (本キット) |
|--------|----------------|
| `urls.py` | `src/main.rs` routes! |
| `views.py` | `src/controllers/*.rs` |
| `models.py` | `src/entities/*.rs` |
| `forms.py` | `#[derive(FromForm)]` 構造体 |
| `templates/` | `templates/*.html.tera` |
| `static/` | `static/` |
| `@login_required` | `AuthenticatedUser` ガード |
| `@staff_member_required` | `AdminUser` ガード |
| `messages.success()` | `Flash::success()` |
| `User.objects.all()` | `User::find().all()` |

---

## 🤝 貢献

プルリクエストを歓迎します！

1. Fork
2. 機能ブランチを作成 (`git checkout -b feature/amazing-feature`)
3. コミット (`git commit -m 'Add amazing feature'`)
4. Push (`git push origin feature/amazing-feature`)
5. Pull Request作成

---

## 📄 ライセンス

MIT License - 詳細は[LICENSE](LICENSE)を参照してください。

---

<div align="center">

**Happy Coding with Rust! 🦀✨**

*Made with ❤️ for Django developers*

</div>
