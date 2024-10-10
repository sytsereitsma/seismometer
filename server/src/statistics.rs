pub struct Statistics {
    pub sum: i64,
    pub min: i32,
    pub max: i32,
    pub count: i64,
}

impl Statistics {
    pub fn new() -> Statistics {
        Statistics {
            sum: 0,
            min: i32::MAX,
            max: i32::MIN,
            count: 0,
        }
    }

    pub fn add(&mut self, value: i32) {
        self.count += 1;
        self.sum += value as i64;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    pub fn mean(&self) -> Option<f64> {
        if self.count == 0 {
            return None;
        }

        Some(self.sum as f64 / self.count as f64)
    }

    pub fn print_and_reset(&mut self, prefix: &str) {
        println!(
            "{} Min: {}, Avg: {}, Max: {}",
            prefix,
            self.min,
            self.mean().unwrap_or(0.0),
            self.max
        );

        self.sum = 0;
        self.min = i32::MAX;
        self.max = i32::MIN;
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::statistics::Statistics;

    #[test]
    fn test_new() {
        let mut stats = Statistics::new();
        assert!(stats.mean().is_none());

        stats.add(123);
        assert_eq!(stats.min, 123);
        assert_eq!(stats.max, 123);
        assert_eq!(stats.sum, 123);
        assert_eq!(stats.count, 1);
        assert_eq!(stats.mean(), Some(123.0));

        stats.add(456);
        assert_eq!(stats.min, 123);
        assert_eq!(stats.max, 456);
        assert_eq!(stats.sum, 579);
        assert_eq!(stats.count, 2);
        assert_eq!(stats.mean(), Some(289.5));

        stats.add(-456);
        assert_eq!(stats.min, -456);
        assert_eq!(stats.max, 456);
        assert_eq!(stats.sum, 123);
        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean(), Some(41.0));
    }
}
