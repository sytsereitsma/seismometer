use record::Record;

use crate::record;

pub struct TriggerDetector {
    threshold: i32,
    window: Vec<Record>,
    index: usize,
    window_full: bool,
    min_trigger_count: usize, // Number of consecutive triggers required to activate the trigger.
}

impl TriggerDetector {
    pub fn new(threshold: i32, window_size: usize) -> TriggerDetector {
        TriggerDetector {
            threshold,
            window: vec![Record::default(); window_size],
            index: 0,
            window_full: false,
            min_trigger_count: 3,
        }
    }

    fn detect_trigger(&self) -> bool {
        let end = if self.window_full {
            self.window.len()
        } else {
            self.index
        };

        let mut min_x = i32::MAX; // Minimal x  in the window
        let mut min_y = i32::MAX; // Minimal y value in the window
        let mut min_z = i32::MAX; // Minimal z value in the window

        // First determine the minimal values in the window
        for i in 0..end {
            let record = &self.window[i];
            min_x = min_x.min(record.x_filt);
            min_y = min_y.min(record.y_filt);
            min_z = min_z.min(record.z_filt);
        }

        let mut triggered = false;
        let mut counts_x = 0; // Number of x triggers in the window
        let mut counts_y = 0; // Number of y triggers in the window
        let mut counts_z = 0; // Number of z triggers in the window

        // Next count the number of values with a difference from the minimal value larger than the threshold
        for i in 0..end {
            let record = &self.window[i];

            counts_x += ((record.x_filt - min_x) > self.threshold) as usize;
            counts_y += ((record.y_filt - min_y) > self.threshold) as usize;
            counts_z += ((record.z_filt - min_z) > self.threshold) as usize;

            triggered = counts_x >= self.min_trigger_count
                || counts_y >= self.min_trigger_count
                || counts_z >= self.min_trigger_count;
            
            if triggered {
                break;
            }
        }

        triggered
    }

    pub fn detect(&mut self, record: &Record) -> bool {
        let prev_index = self.index;
        self.window[self.index] = record.clone();
        self.index = (self.index + 1) % self.window.len();

        if prev_index > self.index {
            self.window_full = true;
        }

        self.detect_trigger()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_record(x: i32, y: i32, z: i32) -> Record {
        let mut record = Record::default();

        record.x_filt = x;
        record.y_filt = y;
        record.z_filt = z;
        record
    }

    #[test]
    fn test_detect_x_triggers() {
        let mut detector = TriggerDetector::new(5, 3);
        detector.min_trigger_count = 1;

        assert_eq!(detector.detect(&test_record(1, 0, 0)), false); // [1, -, -]
        assert_eq!(detector.detect(&test_record(3, 0, 0)), false); // [1, 3, -]
        assert_eq!(detector.detect(&test_record(-3, 0, 0)), true); // [1, 3, -3]

        // The 3 is now the max value in the window, when it is replaced with 2 it should disable the trigger
        assert_eq!(detector.detect(&test_record(2, 0, 0)), true); // [3, -3, 2]
        assert_eq!(detector.detect(&test_record(2, 0, 0)), false); // [2, -3, 2]
    }

    #[test]
    fn test_detect_y_triggers() {
        let mut detector = TriggerDetector::new(5, 3);
        detector.min_trigger_count = 1;

        assert_eq!(detector.detect(&test_record(0, 1, 0)), false); // [1, -, -]
        assert_eq!(detector.detect(&test_record(0, 3, 0)), false); // [1, 3, -]
        assert_eq!(detector.detect(&test_record(0, -3, 0)), true); // [1, 3, -3]

        // The 3 is now the max value in the window, when it is replaced with 2 it should disable the trigger
        assert_eq!(detector.detect(&test_record(0, 2, 0)), true); // [3, -3, 2]
        assert_eq!(detector.detect(&test_record(0, 2, 0)), false); // [2, -3, 2]
    }

    #[test]
    fn test_detect_z_triggers() {
        let mut detector = TriggerDetector::new(5, 3);
        detector.min_trigger_count = 1;

        assert_eq!(detector.detect(&test_record(0, 0, 1)), false); // [1, -, -]
        assert_eq!(detector.detect(&test_record(0, 0, 3)), false); // [1, 3, -]
        assert_eq!(detector.detect(&test_record(0, 0, -3)), true); // [1, 3, -3]

        // The 3 is now the max value in the window, when it is replaced with 2 it should disable the trigger
        assert_eq!(detector.detect(&test_record(0, 0, 2)), true); // [3, -3, 2]
        assert_eq!(detector.detect(&test_record(0, 0, 2)), false); // [2, -3, 2]
    }

    #[test]
    fn test_trigger_when_window_is_not_full_yet() {
        // A trigger should be detected when the window is not full yet.
        let mut detector = TriggerDetector::new(5, 3);
        detector.min_trigger_count = 1;

        // Pick a high number to make sure we do not get a false trigger based on the window's init values
        assert_eq!(detector.detect(&test_record(99, 0, 0)), false);
        assert_eq!(detector.detect(&test_record(93, 0, 0)), true);
    }

    #[test]
    fn test_min_trigger_count() {
        // A trigger should be detected when the window is not full yet.
        const THRESHOLD: i32 = 5;

        let mut detector = TriggerDetector::new(THRESHOLD, 4);
        detector.min_trigger_count = 3;

        assert_eq!(detector.detect(&test_record(0, 0, 0)), false);
        assert_eq!(detector.detect(&test_record(6, 0, 0)), false); // trigger_count == 1
        assert_eq!(detector.detect(&test_record(6, 0, 0)), false); // trigger_count == 2
        assert_eq!(detector.detect(&test_record(6, 0, 0)), true); // trigger_count == 3 -> active
    }

    #[test]
    fn test_min_trigger_count_with_false_trigger() {
        // A trigger should be detected when the window is not full yet.
        const THRESHOLD: i32 = 5;

        let mut detector = TriggerDetector::new(THRESHOLD, 5);
        detector.min_trigger_count = 3;

        assert_eq!(detector.detect(&test_record(0, 0, 0)), false);
        assert_eq!(detector.detect(&test_record(0, THRESHOLD + 1, 0)), false); // trigger_count == 1
        assert_eq!(detector.detect(&test_record(0, THRESHOLD + 1, 0)), false); // trigger_count == 2
        assert_eq!(detector.detect(&test_record(0, THRESHOLD, 0)), false); // trigger_count == 2
        assert_eq!(detector.detect(&test_record(0, THRESHOLD + 1, 0)), true); // trigger_count == 3 -> active
    }
}
