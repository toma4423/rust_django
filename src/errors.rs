use rocket::http::Status;
use rocket::response::{self, Responder, Response};
use rocket::Request;
use sea_orm::DbErr;
use std::io::Cursor;

/// アプリケーション全体で使用するエラー型。
/// Djangoの例外クラス (PermissionDenied, Http404 等) に相当します。
#[derive(Debug)]
pub enum AppError {
    /// データベースエラー
    Database(DbErr),
    /// 認証エラー (401 Unauthorized)
    Unauthorized,
    /// 権限エラー (403 Forbidden)
    Forbidden,
    /// リソースが見つからない (404 Not Found)
    NotFound,
    /// 不正なリクエスト (400 Bad Request)
    BadRequest(String),
    /// 内部エラー (500 Internal Server Error)
    Internal(String),
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (status, message) = match &self {
            AppError::Unauthorized => (Status::Unauthorized, "Unauthorized"),
            AppError::Forbidden => (Status::Forbidden, "Forbidden"),
            AppError::NotFound => (Status::NotFound, "Not Found"),
            AppError::BadRequest(msg) => (Status::BadRequest, msg.as_str()),
            AppError::Database(_) => (Status::InternalServerError, "Database Error"),
            AppError::Internal(msg) => (Status::InternalServerError, msg.as_str()),
        };

        Response::build()
            .status(status)
            .sized_body(message.len(), Cursor::new(message.to_string()))
            .ok()
    }
}

impl From<DbErr> for AppError {
    fn from(e: DbErr) -> Self {
        AppError::Database(e)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Database(e) => write!(f, "Database error: {}", e),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::Forbidden => write!(f, "Forbidden"),
            AppError::NotFound => write!(f, "Not found"),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}
