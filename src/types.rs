use serde::Serialize;

#[derive(Serialize)]
pub struct MessagePreview {
    pub title: String,
    pub author: String,
    pub time: String,
    pub text: String
}