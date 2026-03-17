#[derive(Debug)]
pub enum UserEvent {
    Empty,
    IpcHandle()
}

impl Default for UserEvent {
    fn default() -> Self {
        UserEvent::Empty
    }
}
