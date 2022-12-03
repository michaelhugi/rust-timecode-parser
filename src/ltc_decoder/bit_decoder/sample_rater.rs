/// Bit duration on LTC with 30fps is 416µs. For some margin we estimate that a bit will have
/// a min duration of 405µs
const MIN_BIT_DURATION_S: f64 = (405.0 + 2.0 / 3.0) / 1000000.0;
/// Bit duration on LTC with 24fps is 520µs. For some margin we estimate that a bit will
/// have a max duration of 530µs
const MAX_BIT_DURATION_S: f64 = (530.0 + 2.0 / 3.0) / 1000000.0;

/// Returns the max sample count to expect for a full bit in LTC code
pub(crate) fn max_sample_count_for_bit(sample_rate: &f64) -> usize {
    (sample_rate * MAX_BIT_DURATION_S) as usize +2
}

/// Returns the max sample count to expect for a full bit in LTC code
pub(crate) fn min_sample_count_for_bit(sample_rate: &f64) -> usize {
    (sample_rate * MIN_BIT_DURATION_S) as usize -2
}

/// Returns the max sample count to expect for a full bit in LTC code
pub(crate) fn max_sample_count_for_halfbit(sample_rate: &f64) -> usize {
    (sample_rate * MAX_BIT_DURATION_S / 2.0) as usize +2
}

/// Returns the max sample count to expect for a full bit in LTC code
pub(crate) fn min_sample_count_for_halfbit(sample_rate: &f64) -> usize {
    (sample_rate * MIN_BIT_DURATION_S / 2.0) as usize -2
}