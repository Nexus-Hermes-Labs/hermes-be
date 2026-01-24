use serde::{Deserialize, Serialize};

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: u32,
    pub page_size: u32,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

impl PaginationParams {
    pub fn new(page: u32, page_size: u32) -> Self {
        Self { page, page_size }
    }

    pub fn offset(&self) -> i64 {
        ((self.page - 1) * self.page_size) as i64
    }

    pub fn limit(&self) -> i64 {
        self.page_size as i64
    }
}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

impl<T> Paginated<T> {
    pub fn new(items: Vec<T>, total: i64, params: PaginationParams) -> Self {
        let total_pages = ((total as f64) / (params.page_size as f64)).ceil() as u32;
        Self {
            items,
            total,
            page: params.page,
            page_size: params.page_size,
            total_pages,
        }
    }

    pub fn empty(params: PaginationParams) -> Self {
        Self {
            items: vec![],
            total: 0,
            page: params.page,
            page_size: params.page_size,
            total_pages: 0,
        }
    }
}
