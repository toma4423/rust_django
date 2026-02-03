use sea_orm::*;
use rocket_dyn_templates::{Template, context};
use serde::Serialize;
use rocket::serde::json::serde_json;

/// 汎用的な一覧表示ビューのためのトレイト。
/// Djangoの `ListView` に相当します。
#[rocket::async_trait]
pub trait ListView<E>
where
    E: EntityTrait,
    E::Model: Serialize + Sync + Send,
{
    /// 使用するテンプレート名 (例: "admin/list")
    fn template_name(&self) -> &'static str;

    /// 1ページあたりの表示件数 (Django: paginate_by)
    fn per_page(&self) -> usize {
        10
    }

    /// ベースとなるクエリを取得 (Django: get_queryset)
    fn get_queryset(&self) -> Select<E> {
        E::find()
    }

    /// コンテキストに追加データを注入する (Django: get_context_data)
    fn get_context_data(&self, _db: &DatabaseConnection) -> serde_json::Value {
        serde_json::json!({})
    }

    /// 検索フィルタを適用する hooks
    /// q: 検索クエリ文字列
    fn filter_queryset(&self, query: Select<E>, _q: &str) -> Select<E> {
        query
    }

    /// リスト表示のメイン処理
    async fn list(
        &self,
        db: &DatabaseConnection,
        page: usize,
        q: Option<String>,
        extra_context: serde_json::Value,
    ) -> Template {
        let page = if page < 1 { 1 } else { page };
        // Paginate expects u64
        let per_page = self.per_page() as u64;

        // 1. クエリ構築
        let mut query = self.get_queryset();

        // 2. 検索適用
        let search_query = q.clone().unwrap_or_default();
        if !search_query.trim().is_empty() {
            query = self.filter_queryset(query, &search_query);
        }

        // 3. ページネーション
        let paginator = query.paginate(db, per_page);
        let num_pages = paginator.num_pages().await.unwrap_or(0);
        let items = paginator.fetch_page((page - 1) as u64).await.unwrap_or_default();

        // 4. コンテキスト構築
        // context!マクロは独自型を返すため、一度json Valueに変換してマージ操作を可能にする
        let base_context = context! {
            items: items,
            current_page: page,
            num_pages: num_pages,
            search_query: search_query,
        };
        
        let mut context_value = serde_json::to_value(base_context).unwrap_or(serde_json::json!({}));

        // 追加コンテキストのマージ
        if let serde_json::Value::Object(ref mut map) = context_value {
             // 1. get_context_dataからのデータ
             if let serde_json::Value::Object(static_extra) = self.get_context_data(db) {
                 for (k, v) in static_extra {
                     map.insert(k, v);
                 }
             }
             // 2. 引数で渡された動的データ (CSRFトークンなど)
             if let serde_json::Value::Object(dynamic_extra) = extra_context {
                 for (k, v) in dynamic_extra {
                     map.insert(k, v);
                 }
             }
        }

        Template::render(self.template_name(), context_value)
    }
}
