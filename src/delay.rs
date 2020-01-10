
use std::time::Duration;
use std::thread::sleep;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};



#[derive(Clone)]
pub struct Delay {
    time: Duration,
    serial: Arc<AtomicU64>,
}

impl Delay {
    pub fn new(time: Duration) -> Self {
        Self {
            time,
            serial: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn wait(&self) -> bool {
        let expect = self.serial.fetch_add(1, Ordering::Relaxed) + 1;
        sleep(self.time);
        let actual = self.serial.load(Ordering::Relaxed);
        expect == actual
    }
}
