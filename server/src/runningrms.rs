/// Running RMS of i32 numbers. The result is rounded to nearest u32
pub struct RunningRMS {
    /// Contains squares of the samples, individual squares can be u64, but the sum of all squares must be u128
    window: Vec<u64>,
    /// The sum of all values in the window (i.e. the sum of the squares)
    sqsum: u128,
    /// The index of the next sample to be added
    index: usize,
    /// True if the window is full
    window_full: bool,
}

impl RunningRMS {
    pub fn new(window_size: usize) -> RunningRMS {
        RunningRMS {
            window: vec![0; window_size],
            sqsum: 0,
            index: 0,
            window_full: false,
        }
    }

    /// Add a new sample to the running RMS calculation and return the RMS value if the window is full
    pub fn add_sample(&mut self, value: i32) -> Option<u32> {
        // Subtract the value of the oldest index, which happens to be the one we're overwriting
        self.sqsum -= self.window[self.index] as u128;

        // Add the new sample
        let sq = value as i64 * value as i64;
        self.window[self.index] = sq as u64;
        self.sqsum += sq as u128;

        // Move the index to the next position
        self.index = (self.index + 1) % self.window.len();
        self.window_full = self.window_full || (self.index == 0);

        self.rms()
    }

    fn rms(&self) -> Option<u32> {
        if self.window_full {
            // The square sum may not fit in an f64 and f128 is unstable, so do the division in 2 steps
            // First the integer part, then the fractional part, this way we know the value fits in a u32/f64
            let quotient = self.sqsum / self.window.len() as u128;
            let remainder = self.sqsum % self.window.len() as u128;
            let rms = ((quotient as f64) + (remainder as f64 / self.window.len() as f64)).sqrt();
            Some(rms.round() as u32)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn expected_rms(a: i32, b: i32, c: i32) -> u32 {
        let sqsum = (a as i64 * a as i64) as u128 + (b as i64 * b as i64) as u128 + (c as i64 * c as i64) as u128;

        let quotient = sqsum / 3 as u128;
        let remainder = sqsum % 3 as u128;
        let rms = ((quotient as f64) + (remainder as f64 / 3 as f64)).sqrt();
        
        rms.round() as u32        
    }

    #[test]
    fn running_rms() {
        let mut rms = RunningRMS::new(3);
        
        // Note the RMS is rounded to the nearest integer
        assert_eq!(rms.add_sample(11), None);
        assert_eq!(rms.add_sample(22), None);
        assert_eq!(rms.add_sample(33), Some(expected_rms(11, 22, 33)));
        assert_eq!(rms.add_sample(44), Some(expected_rms(22, 33, 44)));
        assert_eq!(rms.add_sample(55), Some(expected_rms(33, 44, 55)));
        assert_eq!(rms.add_sample(66), Some(expected_rms(44, 55, 66)));
    }

    #[test]
    fn running_rms_negative() {
        let mut rms = RunningRMS::new(3);
        
        // Note the RMS is rounded to the nearest integer
        assert_eq!(rms.add_sample(-11), None);
        assert_eq!(rms.add_sample(-22), None);
        assert_eq!(rms.add_sample(-33), Some(expected_rms(-11, -22, -33)));
        assert_eq!(rms.add_sample(-44), Some(expected_rms(-22, -33, -44)));
        assert_eq!(rms.add_sample(-55), Some(expected_rms(-33, -44, -55)));
        assert_eq!(rms.add_sample(-66), Some(expected_rms(-44, -55, -66)));
    }

    #[test]
    fn big_numbers() {
        let mut rms = RunningRMS::new(3);
        
        // Note the RMS is rounded to the nearest integer
        assert_eq!(rms.add_sample(0x7FFFFFFF), None);
        assert_eq!(rms.add_sample(0x7FFFFFFE), None);
        assert_eq!(rms.add_sample(0x7FFFFFFD), Some(expected_rms(0x7FFFFFFF, 0x7FFFFFFE, 0x7FFFFFFD)));
    }
}
