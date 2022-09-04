use std::time::Duration;

pub struct EditorWindowSettings {
    pub switch_modes_duration: Duration,
}
impl Default for EditorWindowSettings {
    fn default() -> Self {
        Self {
            switch_modes_duration: Duration::from_secs_f64(0.25),
        }
    }
}