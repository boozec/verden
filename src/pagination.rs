use serde::Deserialize;

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i64>,
}
