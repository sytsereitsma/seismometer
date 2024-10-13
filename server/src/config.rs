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
pub struct EventRecorderConfig {
    pub rms_window: usize,         // Number of samples in the RMS circular buffer
    pub rms_delta_window: usize,       // Number of samples in the RMS delta circular buffer
    pub x_trigger_level: i32,      // Trigger level for the X channel
    pub y_trigger_level: i32,      // Trigger level for the Y channel
    pub z_trigger_level: i32,      // Trigger level for the Z channel
    pub pre_trigger_time_ms: u32,  // Time window before the trigger event
    pub post_trigger_time_ms: u32, // Time window after the trigger event
}

impl Default for EventRecorderConfig {
    fn default() -> EventRecorderConfig {
        EventRecorderConfig {
            rms_window: 100,
            rms_delta_window: 20,
            x_trigger_level: 80_000,
            y_trigger_level: 80_000,
            z_trigger_level: 4_000,
            pre_trigger_time_ms: 10_000,
            post_trigger_time_ms: 10_000,
        }
    }
}

#[derive(Deserialize, Default)]
pub struct Config {
    pub port: String,
    pub statistics: StatisticsConfig,
    pub event_recorder: EventRecorderConfig,
}
