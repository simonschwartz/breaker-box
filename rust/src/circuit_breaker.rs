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
	/// Specify the capacity of the ring buffer.
	pub buffer_size: usize,
	/// Define the minimum number of events required in the buffer to evaluate the error rate.
	pub min_eval_size: usize,
	/// Set the error rate percentage that will trigger the circuit to open.
	pub error_threshold: f32,
	/// Specify the duration (in seconds) the circuit breaker remains open before transitioning to half-open.
	pub retry_timeout: Duration,
	/// Determine the duration (in seconds) each node/span in the buffer stores data.
	pub buffer_span_duration: Duration,
	/// Set the number of consecutive successes required to close a half-open circuit.
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
		self.evaluate_state();
		self.state
	}

	pub fn record<T, E>(&mut self, input: Result<T, E>) {
		self.evaluate_state();
		if let State::Open(_) = self.state {
			return;
		}

		if let State::HalfOpen = self.state {
			if input.is_ok() {
				self.trial_success += 1;
				self.evaluate_state();
				return;
			} else {
				self.state = State::Open(Instant::now());
				self.trial_success = 0;
				return;
			}
		}

		if input.is_ok() {
			self.buffer.add_success(self.settings.buffer_span_duration, Instant::now());
		} else {
			self.buffer.add_failure(self.settings.buffer_span_duration, Instant::now());
		}
	}

	pub fn evaluate_state(&mut self) {
		match self.state {
			State::Open(timeout) => {
				if timeout.elapsed() >= self.settings.retry_timeout {
					self.state = State::HalfOpen;
				}
			},
			State::Closed => {
				let _ = self.buffer.get_cursor(self.settings.buffer_span_duration, Instant::now());
			},
			State::HalfOpen => {
				if self.trial_success >= self.settings.trial_success_required {
					self.trial_success = 0;
					self.state = State::Closed;
					self.buffer.reset_start_time();
				}
			},
		}
	}

	pub fn get_buffer(&mut self) -> &mut RingBuffer {
		&mut self.buffer
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
	use crate::ring_buffer::NodeInfo;

	#[test]
	fn new_test() {
		assert_eq!(CircuitBreaker::new(Settings::default()).buffer.get_buffer_size(), 5);
		assert_eq!(CircuitBreaker::new(Settings::default()).settings, Settings::default());
		assert_eq!(
			CircuitBreaker::new(Settings {
				buffer_size: 10,
				..Settings::default()
			})
			.buffer
			.get_buffer_size(),
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
		assert_eq!(CircuitBreaker::new(Settings::default()).get_state(), State::Closed);
	}

	#[test]
	fn record_test() {
		let mut cb = CircuitBreaker::new(Settings::default());
		assert_eq!(
			cb.buffer.get_node_info(0),
			NodeInfo {
				success_count: 0,
				failure_count: 0,
			}
		);
		cb.record::<(), &str>(Ok(()));
		assert_eq!(
			cb.buffer.get_node_info(0),
			NodeInfo {
				success_count: 1,
				failure_count: 0,
			}
		);
		cb.record::<(), &str>(Err(""));
		assert_eq!(
			cb.buffer.get_node_info(0),
			NodeInfo {
				success_count: 1,
				failure_count: 1,
			}
		);

		cb.state = State::Open(Instant::now());
		assert_eq!(
			cb.buffer.get_node_info(0),
			NodeInfo {
				success_count: 1,
				failure_count: 1,
			}
		);
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Err(""));
		assert_eq!(
			cb.buffer.get_node_info(0),
			NodeInfo {
				success_count: 1,
				failure_count: 1,
			}
		);

		cb.state = State::HalfOpen;
		assert_eq!(
			cb.buffer.get_node_info(0),
			NodeInfo {
				success_count: 1,
				failure_count: 1,
			}
		);
		assert_eq!(cb.trial_success, 0);
		cb.record::<(), &str>(Ok(()));
		assert_eq!(
			cb.buffer.get_node_info(0),
			NodeInfo {
				success_count: 1,
				failure_count: 1,
			}
		);
		assert_eq!(cb.trial_success, 1);
		cb.record::<(), &str>(Ok(()));
		assert_eq!(
			cb.buffer.get_node_info(0),
			NodeInfo {
				success_count: 1,
				failure_count: 1,
			}
		);
		assert_eq!(cb.trial_success, 2);
		cb.record::<(), &str>(Err(""));
		assert!(matches!(cb.state, State::Open(_)));
	}

	#[test]
	fn record_timed_test() {
		let mut cb = CircuitBreaker::new(Settings {
			buffer_span_duration: Duration::from_secs(1),
			..Settings::default()
		});

		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		std::thread::sleep(Duration::from_secs(1));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		std::thread::sleep(Duration::from_secs(1));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		std::thread::sleep(Duration::from_secs(1));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		std::thread::sleep(Duration::from_secs(1));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));

		// We skip 3 nodes ahead
		std::thread::sleep(Duration::from_secs(3));
		cb.evaluate_state();

		assert_eq!(cb.buffer.get_node_info(0).success_count, 0); // skipped
		assert_eq!(cb.buffer.get_node_info(1).success_count, 0); // skipped
		assert_eq!(cb.buffer.get_node_info(2).success_count, 0); // current
		assert_eq!(cb.buffer.get_node_info(3).success_count, 3);
		assert_eq!(cb.buffer.get_node_info(4).success_count, 3);
	}

	#[test]
	fn evaluate_state_test() {
		// TODO
	}

	#[test]
	fn get_buffer_test() {
		let mut cb = CircuitBreaker::new(Settings::default());
		assert!(std::ptr::eq(cb.get_buffer(), &mut cb.buffer));
	}

	#[test]
	fn get_trial_success_test() {
		let mut cb = CircuitBreaker::new(Settings::default());
		cb.state = State::HalfOpen;
		assert_eq!(cb.get_trial_success(), 0);
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		assert_eq!(cb.get_trial_success(), 3);
	}

	#[test]
	fn get_settings_test() {
		let cb = CircuitBreaker::new(Settings::default());
		assert_eq!(*cb.get_settings(), Settings::default());

		let settings = Settings {
			buffer_size: 666,
			min_eval_size: 42,
			error_threshold: 5.5,
			retry_timeout: Duration::from_millis(55),
			buffer_span_duration: Duration::from_secs(80),
			trial_success_required: 100,
		};
		let cb = CircuitBreaker::new(settings);
		assert_eq!(*cb.get_settings(), settings);
	}

	#[test]
	fn get_error_rate_test() {
		// TODO
	}
}
