# Rust Django TODO App Walkthrough

## 概要
このTODOアプリは、Rust (Rocket + SeaORM) でDjangoの主要な機能を再現したサンプルアプリケーションです。
以下の機能が含まれています：

- **ユーザー認証**: ログイン、ログアウト、セッション管理
- **管理画面**: ユーザー一覧、作成、編集、削除（Django Admin風UI）
- **TODOリスト**: CRUD操作、優先度設定、完了状態の切り替え、ユーザーごとの分離
- **セキュリティ**: CSRF保護、パスワードハッシュ化 (Argon2)、SQLインジェクション対策
- **UI/UX**: Django Adminライクなデザイン、HTMXを使用したインタラクティブな操作

## セットアップと起動

### 動作環境
- Rust (Cargo)
- PostgreSQL

### 起動方法
データベースがローカルで起動している状態で：

```bash
# データベース作成
createdb rust_django_db

# サーバー起動 (環境変数は適宜変更)
DATABASE_URL=postgresql://user:password@localhost/rust_django_db cargo run
```

Dockerを使用する場合：
```bash
docker compose up -d
```

## 使用方法

### 1. ログイン
`http://localhost:8000/` または `http://localhost:8000/auth/login` にアクセスします。
初期管理者アカウント:
- **Username**: `admin`
- **Password**: `admin`

### 2. 管理画面
`http://localhost:8000/admin` にアクセスします。
- ユーザーの追加、編集、削除が可能です。
- 一般ユーザーと管理者（スタッフ）の権限設定ができます。

### 3. TODOアプリ
`http://localhost:8000/todo` にアクセスします。
- **追加**: 「+ TODO追加」ボタンから新規タスクを作成
- **編集**: 各タスクの「編集」ボタン
- **完了**: チェックボックスをクリックするだけで即時反映（HTMXを使用）
- **削除**: 「削除」ボタン（確認ダイアログあり）

### 機能詳細

#### Djangoライクな機能
- **テンプレート継承**: teraテンプレートエンジンを使用し、`extends`, `block` でレイアウトを継承。
- **フォーム検証**: `validator` クレートを使用し、Django Formsのようなバリデーションを実装。
- **ミドルウェア (Fairing/Guard)**: `AuthenticatedUser`, `CsrfToken` などのガードでリクエストを検証。

#### Rustの利点
- **安全性**: 型システムと所有権モデルによるメモリ安全性。
- **パフォーマンス**: 高速な実行速度と並行処理能力。
