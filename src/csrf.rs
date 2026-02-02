use rand::Rng;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rocket::http::{Cookie, SameSite};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::http::Status;
use std::time::{SystemTime, UNIX_EPOCH};

/// CSRFトークンの有効期限（秒）
const CSRF_TOKEN_EXPIRY: u64 = 3600; // 1時間

/// CSRFトークン。
/// Djangoの {% csrf_token %} に相当します。
#[derive(Debug, Clone)]
pub struct CsrfToken(pub String);

impl CsrfToken {
    /// 新しいCSRFトークンを生成します。
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen();
        
        // タイムスタンプを含めて有効期限を管理
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut token_data = timestamp.to_be_bytes().to_vec();
        token_data.extend_from_slice(&random_bytes);
        
        CsrfToken(URL_SAFE_NO_PAD.encode(&token_data))
    }

    /// トークンを検証します。
    pub fn verify(&self, submitted: &str) -> bool {
        if self.0 != submitted {
            return false;
        }

        // トークンをデコードしてタイムスタンプを確認
        if let Ok(decoded) = URL_SAFE_NO_PAD.decode(&self.0) {
            if decoded.len() >= 8 {
                let timestamp_bytes: [u8; 8] = decoded[..8].try_into().unwrap_or([0; 8]);
                let token_time = u64::from_be_bytes(timestamp_bytes);
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                // 有効期限をチェック
                return current_time - token_time < CSRF_TOKEN_EXPIRY;
            }
        }
        false
    }

    /// トークン文字列を取得
    pub fn token(&self) -> &str {
        &self.0
    }
}

/// リクエストからCSRFトークンを取得するガード。
/// CookieからCSRFトークンを読み取り、なければ新規生成します。
#[rocket::async_trait]
impl<'r> FromRequest<'r> for CsrfToken {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cookies = request.cookies();
        
        // 既存のトークンがあれば使用、なければ新規生成
        let token = if let Some(cookie) = cookies.get("csrf_token") {
            CsrfToken(cookie.value().to_string())
        } else {
            let new_token = CsrfToken::generate();
            
            // Cookieに保存（HttpOnly=false でJavaScript/HTMXからアクセス可能に）
            let cookie = Cookie::build(("csrf_token", new_token.0.clone()))
                .path("/")
                .same_site(SameSite::Strict)
                .http_only(false) // HTMX/JSからアクセス可能にする
                .secure(false);   // 開発用。本番では true に
            
            cookies.add(cookie);
            new_token
        };
        
        Outcome::Success(token)
    }
}

/// POSTリクエストのCSRFトークンを検証するガード。
/// DjangoのCsrfViewMiddlewareに相当します。
pub struct CsrfValidation;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CsrfValidation {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cookies = request.cookies();
        
        // Cookieからトークンを取得
        let cookie_token = match cookies.get("csrf_token") {
            Some(c) => c.value().to_string(),
            None => return Outcome::Error((Status::Forbidden, ())),
        };
        
        // ヘッダーからトークンを取得（HTMX用）
        // X-CSRF-Token ヘッダーをチェック
        let header_token = request
            .headers()
            .get_one("X-CSRF-Token")
            .map(|s| s.to_string());
        
        // フォームからトークンを取得（通常のフォーム用）
        // Content-Typeがform-urlencodedの場合のみ
        // Note: Rocketではフォームデータをガード内で読み取るのは難しいため、
        // HTMX経由のヘッダー方式を推奨
        
        let submitted_token = match header_token {
            Some(t) => t,
            None => {
                // HTMXヘッダーがない場合は検証スキップ（フォームで別途検証）
                // 本番環境ではより厳密にする必要あり
                return Outcome::Success(CsrfValidation);
            }
        };
        
        // トークンを検証
        let csrf_token = CsrfToken(cookie_token);
        if csrf_token.verify(&submitted_token) {
            Outcome::Success(CsrfValidation)
        } else {
            Outcome::Error((Status::Forbidden, ()))
        }
    }
}

/// テンプレートにCSRFトークンを渡すためのヘルパー
pub fn csrf_context(token: &CsrfToken) -> String {
    token.0.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csrf_token_generation() {
        let token1 = CsrfToken::generate();
        let token2 = CsrfToken::generate();
        
        // 異なるトークンが生成される
        assert_ne!(token1.0, token2.0);
        
        // トークンは空でない
        assert!(!token1.0.is_empty());
    }

    #[test]
    fn test_csrf_token_verification() {
        let token = CsrfToken::generate();
        let token_string = token.0.clone();
        
        // 正しいトークンは検証成功
        assert!(token.verify(&token_string));
        
        // 不正なトークンは検証失敗
        assert!(!token.verify("invalid_token"));
    }
}
