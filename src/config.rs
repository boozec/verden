// TODO: Everything here must be a std::env

pub static PAGE_LIMIT: i64 = 20;
pub const MAX_UPLOAD_FILE_SIZE: u64 = 1024 * 1024; // 1 MB
pub const SAVE_FILE_BASE_PATH: &str = "./uploads";
pub const UPLOADS_ENDPOINT: &str = "http://localhost:3000/uploads";
