pub struct EMAFilter {
    alpha: f64,
    beta: f64,
    value: f64,
}

/// Exponential Moving Average filter
impl EMAFilter {
    pub fn from(sample_frequency: f64, cutoff_frequency: f64) -> EMAFilter {
        let alpha = 1.0 - (-2.0 * std::f64::consts::PI * cutoff_frequency / sample_frequency).exp();

        EMAFilter {
            alpha: alpha,
            beta: 1.0 - alpha,
            value: 0.0,
        }
    }

    pub fn add_sample(&mut self, sample: f64) -> f64 {
        self.value = self.alpha * sample + self.beta * self.value;
        self.value
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}
