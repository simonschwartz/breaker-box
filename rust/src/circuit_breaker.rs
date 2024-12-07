use std::time::Duration;

use crate::ring_buffer::RingBuffer;

#[derive(Debug, Clone, Copy)]
pub enum State {
	Closed,
	Open,
	HalfOpen,
}

#[derive(Debug, Clone, Copy)]
pub enum Input {
	Success,
	Failure,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
	min_eval_size: usize,
	error_threshold: f32,
	retry_timeout: Duration,
	buffer_span_duration: Duration,
	trial_success_required: usize,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			min_eval_size: 100,
			error_threshold: 10.0,
			retry_timeout: Duration::from_millis(60000),
			buffer_span_duration: Duration::from_secs(200),
			trial_success_required: 20,
		}
	}
}

#[derive(Debug)]
pub struct CircuitBreaker {
	buffer: RingBuffer,
	state: State,
	trial_success: usize,
	settings: Settings,
}

impl CircuitBreaker {
	pub fn new() -> Self {
		Self {
			buffer: RingBuffer::new(5),
			state: State::Closed,
			trial_success: 0,
			settings: Settings::default(),
		}
	}

	pub fn set_buffer_size(mut self, size: usize) -> Self {
		self.buffer = RingBuffer::new(size);
		self
	}

	pub fn set_min_eval_size(mut self, size: usize) -> Self {
		self.settings.min_eval_size = size;
		self
	}

	pub fn set_error_threshold(mut self, threshold: f32) -> Self {
		self.settings.error_threshold = threshold;
		self
	}

	pub fn set_retry_timeout(mut self, timeout: Duration) -> Self {
		self.settings.retry_timeout = timeout;
		self
	}

	pub fn set_buffer_span_duration(mut self, duration: Duration) -> Self {
		self.settings.buffer_span_duration = duration;
		self
	}

	pub fn set_trial_success_required(mut self, amount: usize) -> Self {
		self.settings.trial_success_required = amount;
		self
	}
}

impl Default for CircuitBreaker {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn settings_test() {
		assert_eq!(CircuitBreaker::new().buffer.get_length(), 5);
		assert_eq!(CircuitBreaker::new().settings, Settings::default());
		assert_eq!(CircuitBreaker::new().set_buffer_size(10).buffer.get_length(), 10);
		assert_eq!(
			CircuitBreaker::new()
				.set_min_eval_size(5)
				.set_error_threshold(99.99)
				.set_retry_timeout(Duration::from_millis(20))
				.set_buffer_span_duration(Duration::from_millis(999))
				.set_trial_success_required(42)
				.settings,
			Settings {
				min_eval_size: 5,
				error_threshold: 99.99,
				retry_timeout: Duration::from_millis(20),
				buffer_span_duration: Duration::from_millis(999),
				trial_success_required: 42,
			}
		);
	}
}
