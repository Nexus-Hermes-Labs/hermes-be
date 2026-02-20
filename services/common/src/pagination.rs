/// Pagination query parameters
#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub page: i64,
    pub page_size: i64,
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
    /// Clamped constructor: page min 1, page_size 1..=100
    pub fn new(page: i64, page_size: i64) -> Self {
        Self {
            page: page.max(1),
            page_size: page_size.clamp(1, 100),
        }
    }

    /// SQL OFFSET = (page - 1) * page_size
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.page_size
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

impl<T> Paginated<T> {
    pub fn new(items: Vec<T>, total: i64, page: i64, page_size: i64) -> Self {
        let total_pages = if page_size > 0 {
            (total as f64 / page_size as f64).ceil() as i64
        } else {
            0
        };

        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }

    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    pub fn has_prev(&self) -> bool {
        self.page > 1
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params_clamping() {
        let params = PaginationParams::new(0, 200);
        assert_eq!(params.page, 1);
        assert_eq!(params.page_size, 100);
    }

    #[test]
    fn test_offset_calculation() {
        let params = PaginationParams::new(3, 20);
        assert_eq!(params.offset(), 40); // (3-1) * 20
    }

    #[test]
    fn test_paginated_total_pages() {
        let paginated: Paginated<i32> = Paginated::new(vec![], 45, 1, 20);
        assert_eq!(paginated.total_pages, 3); // ceil(45/20)
    }

    #[test]
    fn test_paginated_has_next_prev() {
        let p: Paginated<i32> = Paginated::new(vec![], 60, 2, 20);
        assert!(p.has_next()); // page 2, total 3
        assert!(p.has_prev()); // page 2 > 1
    }
}
