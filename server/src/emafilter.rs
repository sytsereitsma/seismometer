pub struct EMAFilter {
    alpha: f64,
    beta: f64,
    value: f64,
    first_sample: bool, // When true the filter value is seeded with the first sample
}

/// Exponential Moving Average filter
impl EMAFilter {
    #[cfg(test)]
    pub fn new(alpha: f64) -> EMAFilter {
        EMAFilter {
            alpha: alpha,
            beta: 1.0 - alpha,
            value: 0.0,
            first_sample: true,
        }
    }

    pub fn from(sample_frequency: f64, cutoff_frequency: f64) -> EMAFilter {
        let alpha = 1.0 - (-2.0 * std::f64::consts::PI * cutoff_frequency / sample_frequency).exp();

        EMAFilter {
            alpha: alpha,
            beta: 1.0 - alpha,
            value: 0.0,
            first_sample: true,
        }
    }

    pub fn add_sample(&mut self, sample: f64) -> f64 {
        if self.first_sample {
            self.value = sample;
            self.first_sample = false;
        }
        else {
            self.value = self.alpha * sample + self.beta * self.value;
        }

        self.value
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ema_filter() {
        let mut filter = EMAFilter::new(0.1);

        assert_eq!(filter.add_sample(1.0), 1.0);
        assert_eq!(filter.add_sample(2.0), 1.1);
        assert_eq!(filter.add_sample(3.0), 1.29);
        assert_eq!(filter.add_sample(4.0), 1.561);
        assert_eq!(filter.add_sample(5.0), 1.9049);
    }

    #[test]
    fn ema_filter_cutoff() {
        const SAMPLE_FREQUENCY : f64 = 1000.0;
        let filter = EMAFilter::from(SAMPLE_FREQUENCY, 10.0);
        assert_eq!(filter.alpha, 0.06089863257570738);
        assert_eq!(filter.beta, 1.0 - filter.alpha);
    }
}
