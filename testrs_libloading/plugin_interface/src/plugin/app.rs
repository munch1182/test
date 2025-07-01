#[derive(Debug, Clone)]
pub struct AppContext {
    app_version: &'static str,
}

impl AppContext {
    pub fn new(app_version: &'static str) -> Self {
        Self { app_version }
    }

    pub fn app_version(&self) -> &str {
        &self.app_version
    }
}
