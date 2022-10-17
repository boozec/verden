use crate::model::models::ModelUser;
use crate::user::models::UserList;
use crate::warning::models::WarningUser;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i64>,
}

#[derive(Serialize)]
pub struct UserPagination {
    pub count: i64,
    pub results: Vec<UserList>,
}

#[derive(Serialize)]
pub struct ModelPagination {
    pub count: i64,
    pub results: Vec<ModelUser>,
}

#[derive(Serialize)]
pub struct WarningPagination {
    pub count: i64,
    pub results: Vec<WarningUser>,
}
