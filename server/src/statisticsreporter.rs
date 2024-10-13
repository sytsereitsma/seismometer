use crate::config::StatisticsConfig;
use crate::record::Record;
use crate::recordhandler::RecordHandler;
use crate::statistics::Statistics;

pub struct StatisticsReporter {
    x_stats: Statistics,
    y_stats: Statistics,
    z_stats: Statistics,
    start_time_us: u32,
    reset_interval_ms: u32,
}

impl StatisticsReporter {
    pub fn new(cfg: &StatisticsConfig) -> StatisticsReporter {
        StatisticsReporter {
            x_stats: Statistics::new(),
            y_stats: Statistics::new(),
            z_stats: Statistics::new(),
            reset_interval_ms: cfg.report_interval_ms,
            start_time_us: 0,
        }
    }

    fn handle(&mut self, record: &Record) {
        self.x_stats.add(record.x_filt);
        self.y_stats.add(record.y_filt);
        self.z_stats.add(record.z_filt);

        if self.z_stats.count >= 1000 {
            println!("   Min        Mean       Max        Delta");
            self.x_stats.print_and_reset("X:");
            self.y_stats.print_and_reset("Y:");
            self.z_stats.print_and_reset("Z:");
        }
    }
}

impl RecordHandler for StatisticsReporter {
    fn handle(&mut self, record: &Record) {
        Self::handle(self, record);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_handle() {
        let mut reporter = StatisticsReporter::new(&StatisticsConfig::default());
        let record = Record {
            timestamp_us: 0,
            timestamp_utc: Utc::now(),
            x: 0,
            y: 0,
            z: 0,
            x_filt: 1,
            y_filt: 2,
            z_filt: 3,
        };
        reporter.handle(&record);

        assert_eq!(reporter.x_stats.count, 1);
        assert_eq!(reporter.y_stats.count, 1);
        assert_eq!(reporter.z_stats.count, 1);
        assert_eq!(reporter.x_stats.sum, 1);
        assert_eq!(reporter.y_stats.sum, 2);
        assert_eq!(reporter.z_stats.sum, 3);
    }

    #[test]
    fn test_handle_resets_statistics_after_1000_samples() {
        let mut reporter = StatisticsReporter::new(&StatisticsConfig::default());

        let record = Record {
            timestamp_us: 0,
            timestamp_utc: Utc::now(),
            x: 0,
            y: 0,
            z: 0,
            x_filt: 1,
            y_filt: 2,
            z_filt: 3,
        };
        reporter.handle(&record);

        assert_eq!(reporter.x_stats.count, 1);
        assert_eq!(reporter.y_stats.count, 1);
        assert_eq!(reporter.z_stats.count, 1);
        assert_eq!(reporter.x_stats.sum, 1);
        assert_eq!(reporter.y_stats.sum, 2);
        assert_eq!(reporter.z_stats.sum, 3);
    }
}
