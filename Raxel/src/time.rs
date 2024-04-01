use std::time::SystemTime;
pub struct Time {
    pub now: SystemTime,
    pub then: SystemTime,
    pub update: f32,
    pub wait: f32,
    pub draw: f32,
    pub dt: f32,
}
impl Default for Time {
    fn default() -> Self {
        Self { now: SystemTime::UNIX_EPOCH, then: SystemTime::UNIX_EPOCH, update: 0.0, wait: 0.0, draw: 0.0, dt: 0.0 }
    }
}
