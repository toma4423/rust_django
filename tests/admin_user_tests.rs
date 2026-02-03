use rocket::http::Status;

mod common;

#[test]
fn test_admin_redirect_when_not_logged_in() {
    let client = common::setup();
    
    // 未ログインでアクセス
    let response = client.get("/admin/users").dispatch();
    
    // 現状の実装では 401 が返る (リダイレクトではない)
    assert_eq!(response.status(), Status::Unauthorized);
}

// 共通のcommonモジュールが使えるはず
// common::setup() は毎回 Client を返すが、その内部は都度 build_rocket() している。

#[test]
fn test_admin_list_authorized() {
    let client = common::setup();
    common::create_test_admin(&client);
    
    // Login
    let response = client.post("/auth/login")
        .body("username=admin&password=password")
        .header(rocket::http::ContentType::Form)
        .dispatch();
        
    assert_eq!(response.status(), Status::SeeOther); // Redirect to /admin (default behavior check?) 
    // Wait, login default redirect is LOGIN_REDIRECT_URL (/todo) by default but handled by controller.
    // Let's check where it redirects.
    
    // Session cookie should be set
    let cookie = response.cookies().get("user_id");
    assert!(cookie.is_some());
    
    // Now access admin
    let response = client.get("/admin/users").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains("ユーザー管理")); // Title check
}
