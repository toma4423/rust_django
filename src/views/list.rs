use sea_orm::*;
use rocket_dyn_templates::context;
use crate::views::app_template::AppTemplate;
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

    /// ソート処理を適用する
    fn apply_sorting(&self, mut query: Select<E>, sort: Option<String>, dir: Option<String>) -> (Select<E>, String, String) {
        let sort_col = sort.unwrap_or_else(|| "id".to_string());
        let direction = dir.unwrap_or_else(|| "desc".to_string());
        
        let order = if direction.to_lowercase() == "asc" {
            Order::Asc
        } else {
            Order::Desc
        };

        // カラム名からカラムを特定してソート適用
        // E::Column は Iterable なので全探索可能
        let mut found = false;
        for col in E::Column::iter() {
            // sea_ormのデフォルトカラム名は snake_case と仮定
            if col.as_str() == sort_col {
                query = query.order_by(col, order.clone());
                found = true;
                break;
            }
        }
        
        // 見つからない場合はデフォルト (id desc)
        if !found {
            // デフォルトフォールバックは実装依存だが、ここでは何もしないか、あるいはIDでソート
            // ただし "id" という文字列が常に正しいとは限らないため、EntityのPrimaryKeyを使うのが本来は正しい
            // ここでは簡易的に、もし指定されたカラムが無効ならデフォルトソートを適用するロジックは実装せず、
            // 既存の query を返す（ただし、SQLエラーにならないように注意）。
            // ユーザー指定が無効な場合、デフォルト順序（例えばID）を適用したいが、
            // ここではシンプルに「マッチしなければソート追加なし（DBデフォルト）」とする。
            // 必要なら self.default_sort() を定義して呼ぶ。
        }

        (query, sort_col, direction)
    }

    /// リスト表示のメイン処理
    async fn list(
        &self,
        db: &DatabaseConnection,
        page: usize,
        q: Option<String>,
        sort: Option<String>,
        dir: Option<String>,
        extra_context: serde_json::Value,
    ) -> AppTemplate {
        let page = if page < 1 { 1 } else { page };
        let per_page = self.per_page() as u64;

        // 1. クエリ構築
        let mut query = self.get_queryset();

        // 2. 検索適用
        let search_query = q.clone().unwrap_or_default();
        if !search_query.trim().is_empty() {
             query = self.filter_queryset(query, &search_query);
        }

        // 3. ソート適用
        let (query, current_sort, current_dir) = self.apply_sorting(query, sort, dir);

        // 4. ページネーション
        let paginator = query.paginate(db, per_page);
        let num_pages = paginator.num_pages().await.unwrap_or(0);
        let items = paginator.fetch_page((page - 1) as u64).await.unwrap_or_default();

        // 5. コンテキスト構築
        let base_context = context! {
            items: items,
            current_page: page,
            num_pages: num_pages,
            search_query: search_query,
            sort: current_sort,
            dir: current_dir,
        };
        
        let mut context_value = serde_json::to_value(base_context).unwrap_or(serde_json::json!({}));

        // 追加コンテキストのマージ
        if let serde_json::Value::Object(ref mut map) = context_value {
             if let serde_json::Value::Object(static_extra) = self.get_context_data(db) {
                 for (k, v) in static_extra {
                     map.insert(k, v);
                 }
             }
             if let serde_json::Value::Object(dynamic_extra) = extra_context {
                 for (k, v) in dynamic_extra {
                     map.insert(k, v);
                 }
             }
        }

        AppTemplate::new(self.template_name(), context_value)
    }
}
