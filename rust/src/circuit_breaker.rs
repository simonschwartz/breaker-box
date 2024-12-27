use std::time::{Duration, Instant};

use crate::ring_buffer::RingBuffer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
	Closed,
	Open(Instant),
	HalfOpen,
}

impl std::fmt::Display for State {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let alt = f.alternate();
		match self {
			State::Closed => {
				if alt {
					write!(f, "│")
				} else {
					write!(f, "Closed     ")
				}
			},
			State::Open(_) => {
				if alt {
					write!(f, "\x1b[0m─")
				} else {
					write!(f, "\x1b[41m Open \x1b[0m     ")
				}
			},
			State::HalfOpen => {
				if alt {
					write!(f, "/")
				} else {
					write!(f, "\x1b[43m Half Open \x1b[0m")
				}
			},
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
	pub buffer_size: usize,
	pub min_eval_size: usize,
	pub error_threshold: f32,
	pub retry_timeout: Duration,
	pub buffer_span_duration: Duration,
	pub trial_success_required: usize,
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

#[derive(Debug, PartialEq)]
pub struct CircuitBreaker {
	buffer: RingBuffer,
	state: State,
	trial_success: usize,
	settings: Settings,
}

impl CircuitBreaker {
	pub fn new(settings: Settings) -> Self {
		Self {
			buffer: RingBuffer::new(settings.buffer_size),
			state: State::Closed,
			trial_success: 0,
			settings,
		}
	}

	pub fn get_state(&mut self) -> State {
		if let State::Open(timeout) = self.state {
			if timeout.elapsed() >= self.settings.retry_timeout {
				self.state = State::HalfOpen;
			}
		}

		self.state
	}

	pub fn record<T, E>(&mut self, input: Result<T, E>) {
		if let State::Open(_) = self.state {
			return;
		}

		if let State::HalfOpen = self.state {
			if input.is_ok() {
				self.trial_success += 1;

				if self.trial_success >= self.settings.trial_success_required {
					self.trial_success = 0;
					self.state = State::Closed;
					self.buffer.next();
				}
				return;
			} else {
				self.state = State::Open(Instant::now());
				self.trial_success = 0;
				return;
			}
		}

		if self.buffer.has_exired(self.settings.buffer_span_duration) {
			let error_rate = self.buffer.get_error_rate(self.settings.min_eval_size);
			if self.state == State::Closed && error_rate > self.settings.error_threshold {
				self.state = State::Open(Instant::now());
				return;
			} else {
				self.buffer.next();
			}
		}

		if input.is_ok() {
			self.buffer.add_success();
		} else {
			self.buffer.add_failure();
		}
	}

	pub fn get_buffer(&self) -> &RingBuffer {
		&self.buffer
	}

	pub fn get_trial_success(&self) -> usize {
		self.trial_success
	}

	pub fn get_settings(&self) -> &Settings {
		&self.settings
	}

	pub fn get_error_rate(&self) -> f32 {
		self.buffer.get_error_rate(self.settings.min_eval_size)
	}
}

impl Default for CircuitBreaker {
	fn default() -> Self {
		Self::new(Settings::default())
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn new_test() {
		assert_eq!(CircuitBreaker::new(Settings::default()).buffer.get_length(), 5);
		assert_eq!(CircuitBreaker::new(Settings::default()).settings, Settings::default());
		assert_eq!(
			CircuitBreaker::new(Settings {
				buffer_size: 10,
				..Settings::default()
			})
			.buffer
			.get_length(),
			10
		);
		assert_eq!(
			CircuitBreaker::new(Settings {
				buffer_size: 666,
				min_eval_size: 5,
				error_threshold: 99.99,
				retry_timeout: Duration::from_millis(20),
				buffer_span_duration: Duration::from_millis(999),
				trial_success_required: 42,
				..Settings::default()
			})
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

	#[test]
	fn get_state_test() {
		// TODO
	}

	#[test]
	fn record_test() {
		// TODO
	}

	#[test]
	fn get_buffer_test() {
		// TODO
	}

	#[test]
	fn get_trial_success_test() {
		// TODO
	}

	#[test]
	fn get_settings_test() {
		// TODO
	}

	#[test]
	fn get_error_rate_test() {
		// TODO
	}
}
