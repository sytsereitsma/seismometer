use crate::record::Record;
use crate::runningrms::RunningRMS;

struct ChannelTriggerDetector {
    running_rms: RunningRMS,
    rms_delta_window: Vec<u32>,
    rms_delta_index: usize,
    rms_delta_buffer_full: bool,
    threshold: u32,
}

impl ChannelTriggerDetector {
    fn new(threshold: u32, rms_window_size: usize, rms_delta_window: usize) -> ChannelTriggerDetector {
        ChannelTriggerDetector {
            running_rms: RunningRMS::new(rms_window_size),
            rms_delta_window: vec![0; rms_delta_window],
            rms_delta_index: 0,
            rms_delta_buffer_full: false,
            threshold: threshold,
        }
    }

    fn add_sample(&mut self, value: i32) -> bool {
        if let Some(rms) = self.running_rms.add_sample(value) {
            self.rms_delta_window[self.rms_delta_index] = rms as u32;
            self.rms_delta_index = (self.rms_delta_index + 1) % self.rms_delta_window.len();
            self.rms_delta_buffer_full = self.rms_delta_buffer_full || (self.rms_delta_index == 0);

            if self.rms_delta_buffer_full {                                
                // Determine min and max in the window
                let mut min: u32 = u32::MAX;
                let mut max: u32 = 0;

                for v in self.rms_delta_window.iter() {
                    let value = *v;

                    if value < min {
                        min = value;
                    }
                    if value > max {
                        max = value;
                    }

                    if (max - min) > self.threshold {
                        return true;
                   }
                }
            }
        }
        
        false
    }
}

pub struct TriggerDetector {
    x_detector: ChannelTriggerDetector,
    y_detector: ChannelTriggerDetector,
    z_detector: ChannelTriggerDetector,
}

impl TriggerDetector {
    pub fn new(
        x_threshold: i32,
        y_threshold: i32,
        z_threshold: i32,
        rms_window_size: usize,
        rms_delta_window_size: usize,
    ) -> TriggerDetector {
        TriggerDetector {

            x_detector: ChannelTriggerDetector::new(x_threshold as u32, rms_window_size, rms_delta_window_size),
            y_detector: ChannelTriggerDetector::new(y_threshold as u32, rms_window_size, rms_delta_window_size),
            z_detector: ChannelTriggerDetector::new(z_threshold as u32, rms_window_size, rms_delta_window_size),
        }
    }

    pub fn detect(&mut self, record: &Record) -> bool {
        let x_triggered = self.x_detector.add_sample(record.x_filt);
        let y_triggered = self.y_detector.add_sample(record.y_filt);
        let z_triggered = self.z_detector.add_sample(record.z_filt);

        x_triggered || y_triggered || z_triggered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // fn test_record(x: i32, y: i32, z: i32) -> Record {
    //     let mut record = Record::default();

    //     record.x_filt = x;
    //     record.y_filt = y;
    //     record.z_filt = z;
    //     record
    // }

    // #[test]
    // fn test_detect_x_triggers() {
    //     const X_THRESHOLD: i32 = 5;
    //     const Y_THRESHOLD: i32 = 9999;
    //     const Z_THRESHOLD: i32 = 9999;
    //     const WINDOW_SIZE: usize = 3;

    //     let mut detector = TriggerDetector::new(X_THRESHOLD, Y_THRESHOLD, Z_THRESHOLD, WINDOW_SIZE);

    //     assert!(!detector.detect(&test_record(0, 0, 0))); // [0, -, -]
    //     assert!(!detector.detect(&test_record(X_THRESHOLD + 1, 0, 0))); // [0, X_THRESHOLD, -]
    //     assert!(detector.detect(&test_record(X_THRESHOLD + 1, 0, 0))); // [0, X_THRESHOLD + 1, X_THRESHOLD + 1]
    // }

    // #[test]
    // fn test_detect_y_triggers() {
    //     const X_THRESHOLD: i32 = 9999;
    //     const Y_THRESHOLD: i32 = 5;
    //     const Z_THRESHOLD: i32 = 9999;
    //     const WINDOW_SIZE: usize = 3;

    //     let mut detector = TriggerDetector::new(X_THRESHOLD, Y_THRESHOLD, Z_THRESHOLD, WINDOW_SIZE);

    //     assert!(!detector.detect(&test_record(X_THRESHOLD + 1, 0, Z_THRESHOLD + 1))); // [0, -, -]
    //     assert!(!detector.detect(&test_record(
    //         X_THRESHOLD + 1,
    //         Y_THRESHOLD + 1,
    //         Z_THRESHOLD + 1
    //     ))); // [0, Y_THRESHOLD, -]
    //     assert!(detector.detect(&test_record(0, Y_THRESHOLD + 1, 0))); // [0, Y_THRESHOLD + 1, Y_THRESHOLD + 1]
    // }

    // #[test]
    // fn test_detect_z_triggers() {
    //     const X_THRESHOLD: i32 = 9999;
    //     const Y_THRESHOLD: i32 = 9999;
    //     const Z_THRESHOLD: i32 = 5;
    //     const WINDOW_SIZE: usize = 3;

    //     let mut detector = TriggerDetector::new(X_THRESHOLD, Y_THRESHOLD, Z_THRESHOLD, WINDOW_SIZE);

    //     assert!(!detector.detect(&test_record(X_THRESHOLD + 1, Y_THRESHOLD + 1, 0))); // [0, -, -]
    //     assert!(!detector.detect(&test_record(
    //         X_THRESHOLD + 1,
    //         Y_THRESHOLD + 1,
    //         Z_THRESHOLD + 1
    //     ))); // [0, Z_THRESHOLD, -]
    //     assert!(detector.detect(&test_record(0, 0, Z_THRESHOLD + 1))); // [0, Z_THRESHOLD + 1, Z_THRESHOLD + 1]
    // }

    // #[test]
    // fn test_detect_trigger_sign_is_handled() {
    //     const THRESHOLD: i32 = 5;

    //     let mut detector = TriggerDetector::new(THRESHOLD, THRESHOLD, THRESHOLD, 3);

    //     assert!(!detector.detect(&test_record(0, 0, 0))); // [0, -, -]
    //     assert!(!detector.detect(&test_record(THRESHOLD + 1, 0, 0))); // [0, THRESHOLD, -]
    //                                                                   // All triggering values should be in the same direction
    //     assert!(!detector.detect(&test_record(-(THRESHOLD + 10), 0, 0))); // [0, THRESHOLD + 1, -(THRESHOLD + 10)]
    // }

    // #[test]
    // fn test_negative_trigger() {
    //     const THRESHOLD: i32 = 5;

    //     let mut detector = TriggerDetector::new(THRESHOLD, THRESHOLD, THRESHOLD, 3);

    //     assert!(!detector.detect(&test_record(0, 0, 0))); // [0, -, -]
    //     assert!(!detector.detect(&test_record(-THRESHOLD, 0, 0))); // [0, -THRESHOLD, -]
    //     assert!(detector.detect(&test_record(-THRESHOLD, 0, 0))); // [0, -THRESHOLD, -THRESHOLD]
    // }

    // #[test]
    // fn test_triggers_are_relative_to_first_value() {
    //     const THRESHOLD: i32 = 5;
    //     const FIRST_VALUE: i32 = 123;

    //     let mut detector = TriggerDetector::new(THRESHOLD, THRESHOLD, THRESHOLD, 3);

    //     assert!(!detector.detect(&test_record(FIRST_VALUE, 0, 0))); // [FIRST_VALUE, -, -]
    //     assert!(!detector.detect(&test_record(THRESHOLD + FIRST_VALUE + 1, 0, 0))); // [0, THRESHOLD + FIRST_VALUE + 1, -]
    //     assert!(detector.detect(&test_record(THRESHOLD + FIRST_VALUE + 1, 0, 0)));
    //     // [0, THRESHOLD + FIRST_VALUE + 1, THRESHOLD + FIRST_VALUE + 1]
    // }

    // #[test]
    // fn test_window_rollover() {
    //     const THRESHOLD: i32 = 5;

    //     let mut detector = TriggerDetector::new(THRESHOLD, THRESHOLD, THRESHOLD, 3);

    //     assert!(!detector.detect(&test_record(0, 0, 0))); // [0, -, -]
    //     assert!(!detector.detect(&test_record(1, 0, 0))); // [0, 1, -]
    //     assert!(!detector.detect(&test_record(2, 0, 0))); // [0, 1, 2]
    //     assert!(!detector.detect(&test_record(2 + THRESHOLD + 1, 0, 0))); // [2 + THRESHOLD + 1, 1*, 2]
    //     assert!(detector.detect(&test_record(2 + THRESHOLD + 1, 0, 0))); // [2 + THRESHOLD + 1, 2 + THRESHOLD + 1, 2*]
    // }
}
