use sea_orm::*;
use rocket_dyn_templates::context;
use crate::views::app_template::AppTemplate;
use serde::Serialize;
use rocket::serde::json::serde_json;

/// フィルタ定義構造体
#[derive(Serialize, Clone)]
pub struct AdminFilter {
    pub label: String,
    pub parameter_name: String,
    pub choices: Vec<(String, String)>, // (value, label)
}

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

    /// フィルタ定義を取得する
    fn get_filters(&self) -> Vec<AdminFilter> {
        Vec::new()
    }

    /// 検索フィルタを適用する hooks
    /// q: 検索クエリ文字列
    fn filter_queryset(&self, query: Select<E>, _q: &str) -> Select<E> {
        query
    }

    /// フィルタパラメータを適用する hooks
    /// params: URLクエリパラメータ (key=value)
    fn apply_filters(&self, query: Select<E>, _params: &std::collections::HashMap<String, String>) -> Select<E> {
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
            // Default to ID if possible, explicitly assuming "id" column exists or implemented via get_queryset default order
        }

        (query, sort_col, direction)
    }

    /// リスト表示のメイン処理
    #[allow(clippy::too_many_arguments)]
    async fn list(
        &self,
        db: &DatabaseConnection,
        page: usize,
        q: Option<String>,
        sort: Option<String>,
        dir: Option<String>,
        filters: &std::collections::HashMap<String, String>,
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

        // 3. フィルタ適用
        query = self.apply_filters(query, filters);

        // 4. ソート適用
        let (query, current_sort, current_dir) = self.apply_sorting(query, sort, dir);

        // 5. ページネーション
        let paginator = query.paginate(db, per_page);
        let num_pages = paginator.num_pages().await.unwrap_or(0);
        let items = paginator.fetch_page((page - 1) as u64).await.unwrap_or_default();

        // フィルタ定義と現在の選択状態を構築
        let defined_filters = self.get_filters();
        let mut active_filters = Vec::new();
        for f in &defined_filters {
            let current_val = filters.get(&f.parameter_name).cloned().unwrap_or_default();
            active_filters.push(serde_json::json!({
                "label": f.label,
                "parameter_name": f.parameter_name,
                "choices": f.choices,
                "current_value": current_val
            }));
        }

        // 6. コンテキスト構築
        let base_context = context! {
            items: items,
            current_page: page,
            num_pages: num_pages,
            search_query: search_query,
            sort: current_sort,
            dir: current_dir,
            admin_filters: active_filters, // フロントエンド描画用
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
