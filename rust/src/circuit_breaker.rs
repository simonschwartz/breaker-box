use std::time::{Duration, Instant};

use crate::ring_buffer::RingBuffer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
	Closed,
	Open(Instant),
	HalfOpen(),
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Settings {
	buffer_size: usize,
	min_eval_size: usize,
	error_threshold: f32,
	retry_timeout: Duration,
	buffer_span_duration: Duration,
	trial_success_required: usize,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			buffer_size: 5,
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
			buffer: RingBuffer::new(Settings::default().buffer_size),
			state: State::Closed,
			trial_success: 0,
			settings: Settings::default(),
		}
	}

	pub fn set_buffer_size(mut self, size: usize) -> Self {
		self.settings.buffer_size = size;
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

	pub fn get_state(mut self) -> State {
		if let State::Open(timeout) = self.state {
			if timeout.elapsed() >= self.settings.retry_timeout {
				self.state = State::HalfOpen();
			}
		}

		self.state
	}

	pub fn clear_buffer(mut self) {
		self.buffer = RingBuffer::new(self.settings.buffer_size)
	}

	pub fn record<T, E>(mut self, input: Result<T, E>) {
		if let State::Open(_) = self.state {
			return;
		}

		// TODO: half open state isn't being recorded at all?
		if let State::HalfOpen() = self.state {
			if input.is_ok() {
				self.trial_success += 1;

				if self.trial_success >= self.settings.trial_success_required {
					self.state = State::Closed;
				}
				return;
			} else {
				self.state = State::Open(Instant::now());
				self.trial_success = 0;
				return;
			}
		}

		if self.buffer.has_exired(self.settings.buffer_span_duration) {
			self.buffer.next();
			// TODO: check error rate
		}
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
				.set_buffer_size(666)
				.set_min_eval_size(5)
				.set_error_threshold(99.99)
				.set_retry_timeout(Duration::from_millis(20))
				.set_buffer_span_duration(Duration::from_millis(999))
				.set_trial_success_required(42)
				.settings,
			Settings {
				buffer_size: 666,
				min_eval_size: 5,
				error_threshold: 99.99,
				retry_timeout: Duration::from_millis(20),
				buffer_span_duration: Duration::from_millis(999),
				trial_success_required: 42,
			}
		);
	}
}
