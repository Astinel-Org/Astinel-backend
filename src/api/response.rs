use serde::Serialize;
use axum::Json;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Json<Self> {
        Json(Self { success: true, data })
    }
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: usize, page: usize, per_page: usize) -> Json<Self> {
        Json(Self { success: true, data, total, page, per_page })
    }
}
