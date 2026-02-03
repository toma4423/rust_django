use rocket::{fairing::{Fairing, Info, Kind}, Data, Request};

use rocket::serde::json::serde_json;
use crate::guards::auth::AuthenticatedUser;
use crate::csrf::CsrfToken;

/// コンテキストプロセッサとしてのFairing。
/// リクエスト処理前に共通データ（ユーザー、CSRFトークンなど）を取得・キャッシュします。
pub struct ContextFairing;

#[rocket::async_trait]
impl Fairing for ContextFairing {
    fn info(&self) -> Info {
        Info {
            name: "Global Context Processor",
            kind: Kind::Request,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        // 1. User Context
        // AuthenticatedUser ガードを実行してキャッシュする
        
        // request.guard() returns Outcome.
        // We can't easily "store" the result of a guard in a way that subsequent guards reuse it automatically 
        // unless the Guard *itself* caches implementation.
        // Our AuthenticatedUser uses `FromRequest`.
        // If we call it here, it will run DB query.
        // We want to store the RESULT in `local_cache`.
        
        let user_outcome = request.guard::<AuthenticatedUser>().await;
        if let rocket::outcome::Outcome::Success(auth_user) = user_outcome {
            // Store as a specific type in cache
            // We store the `User` model wrapper or JSON?
            // Storing JSON is easiest for AppTemplate to use.
            if let Ok(user_json) = serde_json::to_value(&auth_user.user) {
                request.local_cache(|| Some(CachedUser(user_json)));
                request.local_cache(|| Some(CachedIsAdmin(auth_user.user.is_admin)));
            }
        }

        // 2. CSRF Context
        // Run guard
        let csrf_outcome = request.guard::<CsrfToken>().await;
        if let rocket::outcome::Outcome::Success(csrf) = csrf_outcome {
             let token_str = csrf.token().to_string();
             request.local_cache(|| Some(CachedCsrf(token_str)));
        }
    }
}

// キャッシュ用の型
#[derive(Clone)]
pub struct CachedUser(pub serde_json::Value);

#[derive(Clone)]
pub struct CachedIsAdmin(pub bool);

#[derive(Clone)]
pub struct CachedCsrf(pub String);
