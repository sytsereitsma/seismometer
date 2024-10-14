use serde::Deserialize;

#[derive(Deserialize)]
pub struct StatisticsConfig {
    pub report_interval_ms: u32,
    pub enabled: bool,
}

impl Default for StatisticsConfig {
    fn default() -> StatisticsConfig {
        StatisticsConfig {
            report_interval_ms: 10_000,
            enabled: false,
        }
    }
}

#[derive(Deserialize)]
pub struct TriggerDetectorConfig {
    pub delta_window: usize, // Number of samples in the filtered delat samples circular buffer
    pub filter_cutoff_frequency: f64, // Cutoff frequency for the delta samples filter
    pub x_trigger_level: i32, // Trigger level for the X channel
    pub y_trigger_level: i32, // Trigger level for the Y channel
    pub z_trigger_level: i32, // Trigger level for the Z channel
    pub debug_filename: Option<String>, // Optional filename for debug output
}

impl Default for TriggerDetectorConfig {
    fn default() -> TriggerDetectorConfig {
        TriggerDetectorConfig {
            delta_window: 20,
            filter_cutoff_frequency: 1.0,
            x_trigger_level: 20_000,
            y_trigger_level: 20_000,
            z_trigger_level: 4_000,
            debug_filename: Some(String::from("seismodata.txt")),
        }
    }
}

#[derive(Deserialize)]
pub struct EventRecorderConfig {
    pub trigger_config: TriggerDetectorConfig,
    pub pre_trigger_time_ms: u32, // Time window before the trigger event
    pub post_trigger_time_ms: u32, // Time window after the trigger event
    pub filename: String,
}

impl Default for EventRecorderConfig {
    fn default() -> EventRecorderConfig {
        EventRecorderConfig {
            trigger_config: TriggerDetectorConfig::default(),
            pre_trigger_time_ms: 10_000,
            post_trigger_time_ms: 10_000,
            filename: String::from("seismodata.txt"),
        }
    }
}

#[derive(Deserialize)]
pub struct RawDataRecorderConfig {
    pub filename: String,
    pub enabled: bool,
}

impl Default for RawDataRecorderConfig {
    fn default() -> RawDataRecorderConfig {
        Self {
            filename: String::from("raw_data.txt"),
            enabled: false,
        }
    }
}

#[derive(Deserialize, Default)]
pub struct Config {
    pub port: String,
    pub statistics: StatisticsConfig,
    pub event_recorder: EventRecorderConfig,
    pub raw_data_recorder: RawDataRecorderConfig,
}
