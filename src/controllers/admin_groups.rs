use rocket::form::Form;
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_dyn_templates::{Template, context};
use sea_orm::*;
use serde::Deserialize;
use crate::entities::{prelude::*, group};
use crate::guards::auth::AdminUser;
use crate::csrf::CsrfToken;

// グループフォーム
#[derive(FromForm, Deserialize)]
pub struct GroupForm<'r> {
    pub name: &'r str,
    #[field(default = "")]
    pub csrf_token: &'r str,
}

// 一覧
#[get("/groups?<page>&<q>")]
pub async fn list_groups(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    csrf: CsrfToken,
    page: Option<usize>,
    q: Option<String>,
) -> Template {
    let db = db as &DatabaseConnection;
    let page = page.unwrap_or(1);
    let per_page = 10;

    let mut query = Group::find().order_by_asc(group::Column::Id);

    if let Some(ref search_query) = q {
        if !search_query.trim().is_empty() {
            query = query.filter(group::Column::Name.contains(search_query));
        }
    }

    let paginator = query.paginate(db, per_page);
    let num_pages = paginator.num_pages().await.unwrap_or(0);
    let groups = paginator.fetch_page((page - 1) as u64).await.unwrap_or_default();

    Template::render("admin/group_list", context! {
        groups: groups,
        active_nav: "groups",
        csrf_token: csrf.token(),
        current_page: page,
        num_pages: num_pages,
        search_query: q.unwrap_or_default(),
    })
}

// 作成フォーム
#[get("/groups/create")]
pub fn create_group_form(_admin: AdminUser, csrf: CsrfToken) -> Template {
    Template::render("admin/group_form", context! {
        active_nav: "groups",
        csrf_token: csrf.token(),
    })
}

// 作成処理
#[post("/groups/create", data = "<form>")]
pub async fn create_group(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    csrf: CsrfToken,
    form: Form<GroupForm<'_>>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    if !csrf.verify(form.csrf_token) {
        return Err(Flash::error(Redirect::to("/admin/groups/create"), "CSRF検証に失敗しました"));
    }

    if form.name.trim().is_empty() {
        return Err(Flash::error(Redirect::to("/admin/groups/create"), "グループ名は必須です"));
    }

    let new_group = group::ActiveModel {
        name: Set(form.name.to_owned()),
        ..Default::default()
    };

    new_group.insert(db.inner()).await.map_err(|e| {
        if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
            Flash::error(Redirect::to("/admin/groups/create"), "このグループ名は既に使用されています")
        } else {
            Flash::error(Redirect::to("/admin/groups/create"), "グループの作成に失敗しました")
        }
    })?;

    Ok(Flash::success(Redirect::to("/admin/groups"), "グループを正常に追加しました"))
}

// 編集フォーム
#[get("/groups/edit/<id>")]
pub async fn edit_group_form(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    csrf: CsrfToken,
    id: i32,
) -> Result<Template, Flash<Redirect>> {
    let group = Group::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(|_| Flash::error(Redirect::to("/admin/groups"), "グループの取得に失敗しました"))?
        .ok_or_else(|| Flash::error(Redirect::to("/admin/groups"), "グループが見つかりません"))?;

    Ok(Template::render("admin/group_form", context! {
        group: group,
        active_nav: "groups",
        csrf_token: csrf.token(),
    }))
}

// 編集処理
#[post("/groups/edit/<id>", data = "<form>")]
pub async fn edit_group(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    csrf: CsrfToken,
    id: i32,
    form: Form<GroupForm<'_>>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    if !csrf.verify(form.csrf_token) {
        return Err(Flash::error(Redirect::to(format!("/admin/groups/edit/{}", id)), "CSRF検証に失敗しました"));
    }

    let existing_group = Group::find_by_id(id)
        .one(db.inner())
        .await
        .map_err(|_| Flash::error(Redirect::to("/admin/groups"), "グループの取得に失敗しました"))?
        .ok_or_else(|| Flash::error(Redirect::to("/admin/groups"), "グループが見つかりません"))?;

    let mut active_model: group::ActiveModel = existing_group.into();
    active_model.name = Set(form.name.to_owned());

    active_model.update(db.inner()).await.map_err(|e| {
        if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
            Flash::error(Redirect::to(format!("/admin/groups/edit/{}", id)), "このグループ名は既に使用されています")
        } else {
            Flash::error(Redirect::to(format!("/admin/groups/edit/{}", id)), "グループの更新に失敗しました")
        }
    })?;

    Ok(Flash::success(Redirect::to("/admin/groups"), "グループを正常に変更しました"))
}

// 削除処理
#[post("/groups/delete/<id>")]
pub async fn delete_group(
    db: &State<DatabaseConnection>,
    _admin: AdminUser,
    id: i32,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    Group::delete_by_id(id)
        .exec(db.inner())
        .await
        .map_err(|_| Flash::error(Redirect::to("/admin/groups"), "グループの削除に失敗しました"))?;

    Ok(Flash::success(Redirect::to("/admin/groups"), "グループを正常に削除しました"))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        list_groups,
        create_group_form,
        create_group,
        edit_group_form,
        edit_group,
        delete_group
    ]
}
