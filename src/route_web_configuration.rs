use askama::Template;
use axum::response::{Html, IntoResponse};
use reqwest::StatusCode;

use crate::configuration::MAX_SAVE_PER_GAME_INFO;

struct Setting {
    id: String,
    name: String,
    input_type: String,
    label: String,
    required: bool,
    value: String,
    max: Option<String>,
    min: Option<String>,
    step: Option<String>,
    pattern: Option<String>,
    placeholder: Option<String>,
}

struct Category {
    title: String,
    settings: Vec<Setting>,
}

#[derive(Template)]
#[template(path = "configuration.html")]
struct ConfigurationTemplate {
    title: String,
    categories: Vec<Category>,
}

pub async fn configuration_handler() -> Result<impl IntoResponse, (StatusCode, String)> {
    let max_save_per_game_setting = Setting {
        id: MAX_SAVE_PER_GAME_INFO.id.to_string(),
        name: MAX_SAVE_PER_GAME_INFO.name.to_string(),
        input_type: "number".to_string(),
        required: false,
        value: match MAX_SAVE_PER_GAME_INFO.get_value_in_db() {
            Ok(maybe_configuration_form) => match maybe_configuration_form {
                Some(configuration_form) => configuration_form.value,
                None => return Err((StatusCode::NOT_FOUND, "not found".to_string())),
            },
            Err(err) => return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
        },
        max: MAX_SAVE_PER_GAME_INFO.max.map(|max| max.to_string()),
        min: MAX_SAVE_PER_GAME_INFO.min.map(|min| min.to_string()),
        step: MAX_SAVE_PER_GAME_INFO.step.map(|step| step.to_string()),
        pattern: MAX_SAVE_PER_GAME_INFO
            .pattern
            .map(|patern| patern.to_string()),
        placeholder: Some("Number of save".to_string()),
        label: MAX_SAVE_PER_GAME_INFO.name.to_string(),
    };

    let category = Category {
        title: "Saves".to_string(),
        settings: vec![max_save_per_game_setting],
    };

    let template = ConfigurationTemplate {
        title: "Configuration".to_string(),
        categories: vec![category],
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}
