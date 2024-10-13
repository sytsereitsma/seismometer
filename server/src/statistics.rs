pub struct Statistics {
    pub sum: i64,
    pub min: i32,
    pub max: i32,
    pub count: u32,
    pub sqsum: u128,
}

impl Statistics {
    pub fn new() -> Statistics {
        Statistics {
            sum: 0,
            min: i32::MAX,
            max: i32::MIN,
            count: 0,
            sqsum: 0,
        }
    }

    pub fn add(&mut self, value: i32) {
        self.count += 1;
        self.sum += value as i64;
        self.sqsum += ((value as i128) * (value as i128)) as u128;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    pub fn mean(&self) -> Option<f64> {
        if self.count == 0 {
            return None;
        }

        Some((self.sum as f64) / (self.count as f64))
    }

    pub fn rms(&self) -> Option<f64> {
        // The square sum may not fit in an f64 and f128 is unstable, so do the division in 2 steps
        // First the integer part, then the fractional part
        if self.count == 0 {
            return None;
        }

        let quotient = self.sqsum / self.count as u128;
        let remainder = self.sqsum % self.count as u128;

        Some(((quotient as f64) + (remainder as f64 / self.count as f64)).sqrt())
    }

    pub fn print_and_reset(&mut self, prefix: &str) {
        println!(
            "{} {:10} {:10} {:10} {:10} {:10}",
            prefix,
            self.min,
            self.mean().unwrap_or(0.0),
            self.max,
            self.max - self.min,
            self.rms().unwrap_or(0.0)
        );

        self.sum = 0;
        self.sqsum = 0;
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
        assert!(stats.rms().is_none());

        stats.add(123);
        assert_eq!(stats.min, 123);
        assert_eq!(stats.max, 123);
        assert_eq!(stats.sum, 123);
        assert_eq!(stats.count, 1);
        assert_eq!(stats.mean(), Some(123.0));
        assert_eq!(stats.rms(), Some(123.0));

        stats.add(456);
        assert_eq!(stats.min, 123);
        assert_eq!(stats.max, 456);
        assert_eq!(stats.sum, 579);
        assert_eq!(stats.count, 2);
        assert_eq!(stats.mean(), Some(289.5));
        assert_eq!(
            stats.rms(),
            Some(((123 * 123 + 456 * 456) as f64 / 2.0_f64).sqrt())
        );

        stats.add(-456);
        assert_eq!(stats.min, -456);
        assert_eq!(stats.max, 456);
        assert_eq!(stats.sum, 123);
        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean(), Some(41.0));
        assert_eq!(
            stats.rms(),
            Some(((123 * 123 + 456 * 456 + -456 * -456) as f64 / 3.0_f64).sqrt())
        );
    }
}
