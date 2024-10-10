use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub trigger_window: usize,
    pub trigger_level: i32,
    pub averaging_window: usize,
    pub port: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            trigger_window: 5,
            trigger_level: 250000,
            averaging_window: 100,
            port: String::default(),
        }
    }
}
