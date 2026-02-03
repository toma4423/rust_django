use sea_orm::*;
use rocket::response::{Flash, Redirect};
use rocket_dyn_templates::{Template, context};
use rocket::serde::json::serde_json;
use serde::Serialize;

/// 新規作成ビューのためのトレイト。
/// Djangoの `CreateView` に相当します。
#[rocket::async_trait]
pub trait CreateView<A>
where
    A: ActiveModelTrait + Send,
    <A::Entity as EntityTrait>::Model: IntoActiveModel<A> + Sync,
{
    /// 使用するテンプレート名
    fn template_name(&self) -> &'static str {
        "admin/form" // デフォルト
    }

    /// 成功時のリダイレクト先URL
    fn success_url(&self) -> String;

    /// フォームの初期データを取得 (GET用)
    fn get_initial(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    /// 追加コンテキストデータ (GET/POST失敗時用)
    async fn get_context_data(&self, _db: &DatabaseConnection) -> serde_json::Value {
        serde_json::json!({})
    }

    /// 保存処理の実装。
    /// フォームデータを受け取り、ActiveModelを構築して保存するロジックは実装者が記述する。
    /// Djangoの `form_valid` に相当。
    async fn save(&self, db: &DatabaseConnection, data: &serde_json::Value) -> Result<<A::Entity as EntityTrait>::Model, DbErr>;

    /// GETリクエスト: フォーム表示
    async fn get(&self, db: &DatabaseConnection, extra_context: serde_json::Value) -> Template {
        let mut context = context! {
            // 初期データなどをここに埋め込む
        };
        
        let mut context_value = serde_json::to_value(context).unwrap_or(serde_json::json!({}));
        
        if let serde_json::Value::Object(ref mut map) = context_value {
             // 1. get_context_data
             if let serde_json::Value::Object(extra) = self.get_context_data(db).await {
                 for (k, v) in extra {
                     map.insert(k, v);
                 }
             }
             // 2. initial data
             if let serde_json::Value::Object(initial) = self.get_initial() {
                  for (k, v) in initial {
                      map.insert(k, v);
                  }
             }
             // 3. extra_context
             if let serde_json::Value::Object(dynamic_extra) = extra_context {
                 for (k, v) in dynamic_extra {
                     map.insert(k, v);
                 }
             }
        }

        Template::render(self.template_name(), context_value)
    }

    /// POSTリクエスト: 保存処理
    /// form_data は フォームから受け取ったデータを JSON Value などに変換したもの、あるいは構造体ラッパー
    async fn post(
        &self,
        db: &DatabaseConnection,
        form_data: &serde_json::Value,
        extra_context: serde_json::Value,
    ) -> Result<Flash<Redirect>, Template> {
        match self.save(db, form_data).await {
            Ok(_) => {
                Ok(Flash::success(Redirect::to(self.success_url()), "作成しました"))
            }
            Err(e) => {
                // エラー時はフォームを再表示
                // ここでエラーメッセージをコンテキストに含めたい
                let mut context_value = serde_json::json!({
                    "error": e.to_string(),
                    "form": form_data, // 入力値を戻す
                });

                if let serde_json::Value::Object(ref mut map) = context_value {
                    if let serde_json::Value::Object(extra) = self.get_context_data(db).await {
                        for (k, v) in extra {
                            map.insert(k, v);
                        }
                    }
                    if let serde_json::Value::Object(dynamic_extra) = extra_context {
                         for (k, v) in dynamic_extra {
                             map.insert(k, v);
                         }
                    }
                }
                
                Err(Template::render(self.template_name(), context_value))
            }
        }
    }
}

/// 更新ビューのためのトレイト。
/// Djangoの `UpdateView` に相当します。
#[rocket::async_trait]
pub trait UpdateView<A>
where
    A: ActiveModelTrait + Send,
    <A::Entity as EntityTrait>::Model: IntoActiveModel<A> + Sync + Serialize,
{
    fn template_name(&self) -> &'static str {
        "admin/form"
    }

    fn success_url(&self) -> String;

    async fn get_context_data(&self, _db: &DatabaseConnection) -> serde_json::Value {
        serde_json::json!({})
    }

    /// IDからモデルを取得する
    async fn get_object(&self, _db: &DatabaseConnection, _id: i32) -> Result<Option<<A::Entity as EntityTrait>::Model>, DbErr> {
         // デフォルト実装は難しい（Entityを知る必要があるため）。
         // 実装先で定義してもらうか、EntityTraitをジェネリクスに含める必要がある。
         Err(DbErr::Custom("Not implemented".to_owned()))
    }

    /// 保存処理
    async fn save(&self, db: &DatabaseConnection, id: i32, data: &serde_json::Value) -> Result<<A::Entity as EntityTrait>::Model, DbErr>;

    /// GET: 編集フォーム表示
    async fn get(
        &self, 
        db: &DatabaseConnection, 
        id: i32, 
        object: Option<<A::Entity as EntityTrait>::Model>,
        extra_context: serde_json::Value
    ) -> Result<Template, Flash<Redirect>> {
        let model = match object {
            Some(m) => m,
            None => return Err(Flash::error(Redirect::to(self.success_url()), "Object not found")),
        };

        // モデルをJSONに変換してフォーム初期値とする
        let initial = serde_json::to_value(&model).unwrap_or(serde_json::json!({}));

        let mut context_value = serde_json::json!({
            "form": initial, // 既存データをフォームに埋め込む
            "is_edit": true,
            "id": id,
        });

        if let serde_json::Value::Object(ref mut map) = context_value {
             if let serde_json::Value::Object(extra) = self.get_context_data(db).await {
                 for (k, v) in extra {
                     map.insert(k, v);
                 }
             }
             if let serde_json::Value::Object(dynamic_extra) = extra_context {
                 for (k, v) in dynamic_extra {
                     map.insert(k, v);
                 }
             }
        }

        Ok(Template::render(self.template_name(), context_value))
    }

    /// POST: 更新実行
    async fn post(
        &self,
        db: &DatabaseConnection,
        id: i32,
        form_data: &serde_json::Value,
        extra_context: serde_json::Value,
    ) -> Result<Flash<Redirect>, Template> {
        match self.save(db, id, form_data).await {
            Ok(_) => Ok(Flash::success(Redirect::to(self.success_url()), "更新しました")),
            Err(e) => {
                 let mut context_value = serde_json::json!({
                    "error": e.to_string(),
                    "form": form_data,
                    "is_edit": true,
                    "id": id,
                });

                if let serde_json::Value::Object(ref mut map) = context_value {
                    if let serde_json::Value::Object(extra) = self.get_context_data(db).await {
                        for (k, v) in extra {
                            map.insert(k, v);
                        }
                    }
                    if let serde_json::Value::Object(dynamic_extra) = extra_context {
                         for (k, v) in dynamic_extra {
                             map.insert(k, v);
                         }
                    }
                }
                
                Err(Template::render(self.template_name(), context_value))
            }
        }
    }
}

/// 削除ビューのためのトレイト
/// Djangoの `DeleteView` に相当
#[rocket::async_trait]
pub trait DeleteView<E>
where
    E: EntityTrait + Send,
    E::Model: IntoActiveModel<E::ActiveModel> + Sync,
    <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: From<i32>,
{
    fn success_url(&self) -> String;

    /// 削除実行
    async fn delete(&self, db: &DatabaseConnection, id: i32) -> Result<DeleteResult, DbErr> {
        E::delete_by_id(id).exec(db).await
    }

    /// POST: 削除処理 (通常は確認画面なしでPOSTで削除、あるいは確認画面を挟むが今回は簡易版)
    async fn post(&self, db: &DatabaseConnection, id: i32) -> Result<Flash<Redirect>, Flash<Redirect>> {
        match self.delete(db, id).await {
            Ok(_) => Ok(Flash::success(Redirect::to(self.success_url()), "削除しました")),
            Err(e) => Err(Flash::error(Redirect::to(self.success_url()), format!("削除失敗: {}", e))),
        }
    }
}
