use std::borrow::Cow;
use rocket::request::Request;
use rocket::response::{Responder, Result};
use rocket::serde::json::serde_json;
use rocket_dyn_templates::Template;
use crate::fairings::context::{CachedUser, CachedIsAdmin, CachedCsrf};

/// アプリケーション標準のテンプレートレスポンダー。
/// Djangoの `context_processors` のように、共通のコンテキスト（ユーザー情報、CSRFトークンなど）を自動注入します。
pub struct AppTemplate {
    pub name: Cow<'static, str>,
    pub context: serde_json::Value,
}

impl AppTemplate {
    pub fn new<N, C>(name: N, context: C) -> Self
    where
        N: Into<Cow<'static, str>>,
        C: serde::Serialize,
    {
        AppTemplate {
            name: name.into(),
            context: serde_json::to_value(context).unwrap_or(serde_json::json!({})),
        }
    }
}

// Rocket 0.5のResponderは同期メソッド（Resultを返す）
// async-trait属性は不要（Templateがそうであるように）
impl<'r> Responder<'r, 'static> for AppTemplate {
    fn respond_to(self, request: &'r Request<'_>) -> Result<'static> {
        // 1. グローバルコンテキストの準備
        let mut global_context = serde_json::Map::new();

        // User Context (ContextFairingでキャッシュ済み)
        if let Some(cached_user) = request.local_cache(|| None::<CachedUser>) {
             global_context.insert("user".into(), cached_user.0.clone());
        }
        if let Some(cached_is_admin) = request.local_cache(|| None::<CachedIsAdmin>) {
             global_context.insert("is_admin".into(), serde_json::Value::Bool(cached_is_admin.0));
        }

        // CSRF Context (ContextFairingでキャッシュ済み)
        if let Some(cached_csrf) = request.local_cache(|| None::<CachedCsrf>) {
             global_context.insert("csrf_token".into(), serde_json::Value::String(cached_csrf.0.clone()));
        }

        // Active Nav Logic (Simple URI check)
        // Manual override possible
        let uri = request.uri().path();
        // default active_nav based on path?
        // e.g. /admin/users -> "users"
        // But manual override is common.

        // 2. マージ (Local Context overrides Global)
        let mut final_context = global_context;
        
        if let serde_json::Value::Object(local_map) = self.context {
            for (k, v) in local_map {
                final_context.insert(k, v);
            }
        }
        
        // 3. Templateに委譲
        let template = Template::render(self.name, serde_json::Value::Object(final_context));
        template.respond_to(request)
    }
}
