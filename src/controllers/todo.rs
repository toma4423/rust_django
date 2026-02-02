use rocket::form::Form;
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_dyn_templates::{Template, context};
use sea_orm::*;
use serde::Deserialize;
use chrono::Utc;
use crate::entities::{prelude::*, todo, group};
use crate::guards::auth::AuthenticatedUser;
use crate::csrf::CsrfToken;

/// TODOフォームのデータ構造
/// Djangoの `forms.ModelForm` に相当
#[derive(FromForm, Deserialize)]
pub struct TodoForm<'r> {
    pub title: &'r str,
    #[field(default = "")]
    pub description: &'r str,
    #[field(default = 1)]
    pub priority: i32,
    #[field(default = false)]
    pub completed: bool,
    #[field(default = "")]
    pub csrf_token: &'r str,
    pub group_id: Option<i32>,
}

/// TODO一覧を表示。
/// Djangoの `ListView` に相当します。
#[get("/")]
pub async fn list_todos(
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
    csrf: CsrfToken,
) -> Template {
    // 現在のユーザーのTODOのみ取得（Djangoの `Todo.objects.filter(user=request.user)`）
    let todos = Todo::find()
        .filter(todo::Column::UserId.eq(user.user.id))
        .find_also_related(Group)
        .order_by_desc(todo::Column::Priority)
        .order_by_asc(todo::Column::Completed)
        .all(db.inner())
        .await
        .unwrap_or_default();

    // テンプレートで扱いやすいように変換せずにそのまま渡す (Tera側で分解)
    // todos: Vec<(todo::Model, Option<group::Model>)>

    Template::render("todo/list", context! {
        todos: todos,
        username: user.user.username.clone(),
        csrf_token: csrf.token(),
    })
}

/// TODO作成フォーム (GET)
#[get("/create")]
pub async fn create_todo_form(db: &State<DatabaseConnection>, user: AuthenticatedUser, csrf: CsrfToken) -> Template {
    // 所属グループを取得
    let groups = user.user.find_related(Group).all(db.inner()).await.unwrap_or_default();
    
    Template::render("todo/form", context! {
        username: user.user.username.clone(),
        csrf_token: csrf.token(),
        groups: groups,
    })
}

/// TODO作成処理 (POST)
#[post("/create", data = "<form>")]
pub async fn create_todo(
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
    csrf: CsrfToken,
    form: Form<TodoForm<'_>>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    // CSRF検証
    if !csrf.verify(form.csrf_token) {
        return Err(Flash::error(Redirect::to("/todo/create"), "CSRF検証に失敗しました"));
    }

    // バリデーション
    if form.title.trim().is_empty() {
        return Err(Flash::error(Redirect::to("/todo/create"), "タイトルは必須です"));
    }

    let now = Utc::now().into();
    
    let new_todo = todo::ActiveModel {
        title: Set(form.title.to_owned()),
        description: Set(if form.description.is_empty() { None } else { Some(form.description.to_owned()) }),
        priority: Set(form.priority),
        completed: Set(form.completed),
        user_id: Set(user.user.id),
        group_id: Set(form.group_id),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    new_todo
        .insert(db.inner())
        .await
        .map_err(|_| Flash::error(Redirect::to("/todo/create"), "TODOの作成に失敗しました"))?;

    Ok(Flash::success(Redirect::to("/todo"), "TODOを作成しました"))
}

/// TODO編集フォーム (GET)
#[get("/edit/<id>")]
pub async fn edit_todo_form(
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
    csrf: CsrfToken,
    id: i32,
) -> Result<Template, Flash<Redirect>> {
    let todo_item = Todo::find_by_id(id)
        .filter(todo::Column::UserId.eq(user.user.id)) // 自分のTODOのみ
        .one(db.inner())
        .await
        .map_err(|_| Flash::error(Redirect::to("/todo"), "TODOの取得に失敗しました"))?
        .ok_or_else(|| Flash::error(Redirect::to("/todo"), "TODOが見つかりません"))?;

    let groups = user.user.find_related(Group).all(db.inner()).await.unwrap_or_default();

    Ok(Template::render("todo/form", context! {
        todo: todo_item,
        username: user.user.username.clone(),
        csrf_token: csrf.token(),
        groups: groups,
    }))
}

/// TODO編集処理 (POST)
#[post("/edit/<id>", data = "<form>")]
pub async fn edit_todo(
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
    csrf: CsrfToken,
    id: i32,
    form: Form<TodoForm<'_>>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    // CSRF検証
    if !csrf.verify(form.csrf_token) {
        return Err(Flash::error(Redirect::to(format!("/todo/edit/{}", id)), "CSRF検証に失敗しました"));
    }

    // 既存のTODOを取得（自分のもののみ）
    let existing = Todo::find_by_id(id)
        .filter(todo::Column::UserId.eq(user.user.id))
        .one(db.inner())
        .await
        .map_err(|_| Flash::error(Redirect::to("/todo"), "TODOの取得に失敗しました"))?
        .ok_or_else(|| Flash::error(Redirect::to("/todo"), "TODOが見つかりません"))?;

    // バリデーション
    if form.title.trim().is_empty() {
        return Err(Flash::error(Redirect::to(format!("/todo/edit/{}", id)), "タイトルは必須です"));
    }

    let mut active_model: todo::ActiveModel = existing.into();
    active_model.title = Set(form.title.to_owned());
    active_model.description = Set(if form.description.is_empty() { None } else { Some(form.description.to_owned()) });
    active_model.priority = Set(form.priority);
    active_model.completed = Set(form.completed);
    active_model.group_id = Set(form.group_id);
    active_model.updated_at = Set(Utc::now().into());

    active_model
        .update(db.inner())
        .await
        .map_err(|_| Flash::error(Redirect::to(format!("/todo/edit/{}", id)), "TODOの更新に失敗しました"))?;

    Ok(Flash::success(Redirect::to("/todo"), "TODOを更新しました"))
}

/// TODO完了/未完了切り替え (HTMX用)
#[post("/toggle/<id>")]
pub async fn toggle_todo(
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
    id: i32,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let existing = Todo::find_by_id(id)
        .filter(todo::Column::UserId.eq(user.user.id))
        .one(db.inner())
        .await
        .map_err(|_| Flash::error(Redirect::to("/todo"), "TODOの取得に失敗しました"))?
        .ok_or_else(|| Flash::error(Redirect::to("/todo"), "TODOが見つかりません"))?;

    let mut active_model: todo::ActiveModel = existing.clone().into();
    active_model.completed = Set(!existing.completed);
    active_model.updated_at = Set(Utc::now().into());

    active_model
        .update(db.inner())
        .await
        .map_err(|_| Flash::error(Redirect::to("/todo"), "更新に失敗しました"))?;

    Ok(Flash::success(Redirect::to("/todo"), 
        if existing.completed { "未完了に戻しました" } else { "完了しました" }))
}

/// TODO削除処理 (POST)
#[post("/delete/<id>")]
pub async fn delete_todo(
    db: &State<DatabaseConnection>,
    user: AuthenticatedUser,
    id: i32,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    // 自分のTODOのみ削除可能
    let result = Todo::delete_many()
        .filter(todo::Column::Id.eq(id))
        .filter(todo::Column::UserId.eq(user.user.id))
        .exec(db.inner())
        .await
        .map_err(|_| Flash::error(Redirect::to("/todo"), "削除に失敗しました"))?;

    if result.rows_affected == 0 {
        return Err(Flash::error(Redirect::to("/todo"), "TODOが見つかりません"));
    }

    Ok(Flash::success(Redirect::to("/todo"), "TODOを削除しました"))
}
