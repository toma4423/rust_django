use validator::{Validate, ValidationError};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Djangoのusernameバリデーション正規表現
    /// 半角英数字、@/./+/-/_ のみ許可
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[\w.@+-]+$").unwrap();
}

/// ユーザー作成/編集フォームのバリデーション。
/// Djangoの `forms.ModelForm` + `clean_*` メソッドに相当。
#[derive(Debug, Validate)]
pub struct UserFormValidation {
    #[validate(
        length(min = 1, max = 150, message = "ユーザー名は1〜150文字の間で入力してください"),
        custom(function = "validate_username_chars", message = "ユーザー名には半角英数字、@/./+/-/_ のみ使用できます")
    )]
    pub username: String,

    #[validate(length(min = 8, message = "パスワードは8文字以上で入力してください"))]
    pub password: Option<String>,
}

/// ユーザー名の文字種バリデーション
fn validate_username_chars(username: &str) -> Result<(), ValidationError> {
    if USERNAME_REGEX.is_match(username) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_username_chars"))
    }
}

impl UserFormValidation {
    /// フォームデータからバリデーション用構造体を作成
    pub fn new(username: &str, password: Option<&str>) -> Self {
        Self {
            username: username.to_string(),
            password: password.map(|p| p.to_string()),
        }
    }

    /// バリデーションを実行し、エラーメッセージを返す
    pub fn validate_form(&self) -> Result<(), Vec<String>> {
        match self.validate() {
            Ok(_) => Ok(()),
            Err(errors) => {
                let mut messages = Vec::new();
                for (field, field_errors) in errors.field_errors() {
                    for error in field_errors {
                        let msg = error.message.as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| format!("{} が不正です", field));
                        messages.push(msg);
                    }
                }
                Err(messages)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_username() {
        let form = UserFormValidation::new("valid_user123", Some("password123"));
        assert!(form.validate_form().is_ok());
    }

    #[test]
    fn test_username_with_special_chars() {
        let form = UserFormValidation::new("user@example.com", Some("password123"));
        assert!(form.validate_form().is_ok());
    }

    #[test]
    fn test_empty_username() {
        let form = UserFormValidation::new("", Some("password123"));
        assert!(form.validate_form().is_err());
    }

    #[test]
    fn test_username_too_long() {
        let long_username = "a".repeat(151);
        let form = UserFormValidation::new(&long_username, Some("password123"));
        assert!(form.validate_form().is_err());
    }

    #[test]
    fn test_password_too_short() {
        let form = UserFormValidation::new("validuser", Some("short"));
        assert!(form.validate_form().is_err());
    }

    #[test]
    fn test_password_optional_for_edit() {
        // 編集時はパスワード省略可能
        let form = UserFormValidation::new("validuser", None);
        assert!(form.validate_form().is_ok());
    }
}
