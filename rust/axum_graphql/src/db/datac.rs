use async_graphql::InputObject;

#[derive(InputObject, Debug)]
pub struct PaginationInput {
    skip: Option<i32>,
    limit: Option<i32>,
}

impl Default for PaginationInput {
    fn default() -> Self {
        Self::new()
    }
}

impl PaginationInput {
    pub fn new() -> Self {
        Self {
            skip: Some(0),
            limit: Some(10),
        }
    }
    pub fn skip(&self) -> i32 {
        self.skip.unwrap_or(0)
    }
    pub fn limit(&self) -> i32 {
        self.limit.unwrap_or(10).clamp(1, 100)
    }
}
