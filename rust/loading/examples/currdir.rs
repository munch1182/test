fn main() {}

#[unsafe(no_mangle)]
pub fn currdir(path: &str) -> String {
    match std::env::current_dir() {
        Ok(mut p) => {
            p.push(path);
            p.to_str().unwrap_or_default().to_string()
        }
        Err(_) => String::default(),
    }
}
