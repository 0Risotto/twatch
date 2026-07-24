use std::sync::Mutex;
use twatch::traits::PlayerService;

pub struct MockPlayerService {
    pub last_url: Mutex<Option<String>>,
    pub last_title: Mutex<Option<String>>,
    pub killed: Mutex<bool>,
}

impl MockPlayerService {
    pub fn new() -> Self {
        Self { last_url: Mutex::new(None), last_title: Mutex::new(None), killed: Mutex::new(false) }
    }
}

impl PlayerService for MockPlayerService {
    fn play(&self, url: &str, title: &str) {
        *self.last_url.lock().unwrap() = Some(url.to_string());
        *self.last_title.lock().unwrap() = Some(title.to_string());
    }

    fn is_running(&self) -> bool {
        self.last_url.lock().unwrap().is_some()
    }

    fn kill(&self) {
        *self.killed.lock().unwrap() = true;
        self.last_url.lock().unwrap().take();
    }
}
