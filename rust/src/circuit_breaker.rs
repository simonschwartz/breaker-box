//! This is the main circuit breaker implementation
//! It allows you to give your system a break when a threshhold of errors has
//! been reached.
use std::time::{Duration, Instant};

use crate::ring_buffer::RingBuffer;

/// The state of our [CircuitBreaker]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
	/// A closed [CircuitBreaker] means requests should be allowed through
	Closed,
	/// An open [CircuitBreaker] means requests should be blocked
	Open(Instant),
	/// A half open [CircuitBreaker] means we count requests until we either have
	/// `Settings.trial_success_required` successful requests, which closes the
	/// circuit or a single failed request which opens it
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

/// The possible settings for our [CircuitBreaker]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
	/// Specify the capacity of the ring buffer
	pub buffer_size: usize,
	/// Determine the duration (in seconds) each node/span in the buffer stores
	/// data
	pub buffer_span_duration: Duration,
	/// Define the minimum number of events required in the buffer to evaluate the
	/// error rate
	pub min_eval_size: usize,
	/// Set the error rate percentage that will trigger the circuit to open
	pub error_threshold: f32,
	/// Specify the duration (in seconds) the [CircuitBreaker] remains open before
	/// transitioning to half-open
	pub retry_timeout: Duration,
	/// Set the number of consecutive successes required to close a half-open
	/// circuit
	pub trial_success_required: usize,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			buffer_size: 5,
			buffer_span_duration: Duration::from_secs(200),
			min_eval_size: 100,
			error_threshold: 10.0,
			retry_timeout: Duration::from_millis(60000),
			trial_success_required: 20,
		}
	}
}

/// The main circuit breaker struct
#[derive(Debug, PartialEq)]
pub struct CircuitBreaker {
	/// The ring buffer for storing failures/successes
	buffer: RingBuffer,
	/// The current state of the [CircuitBreaker]
	state: State,
	/// The last time we recorded something. Used for time-based advancement
	last_record: Instant,
	/// The time when we started (useful for resetting, etc.)
	start_time: Instant,
	/// Consecutive successes when in HalfOpen state
	trial_success: usize,
	/// All relevant circuit-breaker settings in one struct
	settings: Settings,
}

impl CircuitBreaker {
	/// Create a new [CircuitBreaker] with [Settings]
	pub fn new(settings: Settings) -> Self {
		Self {
			buffer: RingBuffer::new(settings.buffer_size),
			state: State::Closed,
			last_record: Instant::now(),
			start_time: Instant::now(),
			trial_success: 0,
			settings,
		}
	}

	/// Get the current state, possibly updating it first if in Open or Closed
	pub fn get_state(&mut self) -> State {
		if let State::Open(_) | State::Closed = self.state {
			self.evaluate_state();
		}

		self.state
	}

	/// Determine if we need to advance the ring buffer based on how much time has
	/// passed since `self.last_record`
	pub fn advance_buffer_for_time(&mut self, now: Instant) {
		let elapsed = now.duration_since(self.last_record);
		if elapsed.is_zero() {
			return;
		}

		let spans_elapsed = elapsed.as_nanos() / self.settings.buffer_span_duration.as_nanos();
		if spans_elapsed > 0 {
			self.buffer.advance(spans_elapsed as usize);
			self.last_record = now;
		}
	}

	/// Record the result of a request: either as a success or failure
	pub fn record<T, E>(&mut self, input: Result<T, E>) {
		if let State::Open(_) | State::Closed = self.state {
			self.evaluate_state();
		}

		match self.state {
			State::Open(_) => {
				// We do not record anything if the circuit is open
			},
			State::HalfOpen => {
				if input.is_ok() {
					self.trial_success += 1;
					self.evaluate_state();
				} else {
					self.state = State::Open(Instant::now());
					self.trial_success = 0;
				}
			},
			State::Closed => {
				self.advance_buffer_for_time(Instant::now());
				if input.is_ok() {
					self.buffer.add_success();
				} else {
					self.buffer.add_failure();
				}
			},
		}
	}

	/// Evaluate and possibly transition the state machine
	pub fn evaluate_state(&mut self) {
		match self.state {
			State::Open(opened_at) => {
				if opened_at.elapsed() >= self.settings.retry_timeout {
					self.state = State::HalfOpen;
				}
			},
			State::Closed => {
				self.advance_buffer_for_time(Instant::now());
				if self.buffer.get_error_rate(self.settings.min_eval_size) > self.settings.error_threshold {
					self.state = State::Open(Instant::now());
				}
			},
			State::HalfOpen => {
				if self.trial_success >= self.settings.trial_success_required {
					self.trial_success = 0;
					self.state = State::Closed;
					// TODO: keep data for more granular error detection
					self.buffer = RingBuffer::new(self.settings.buffer_size);
					self.last_record = Instant::now();
					self.start_time = Instant::now();
				}
			},
		}
	}

	/// Get the ring buffer instance as mutable reference
	pub fn get_buffer(&mut self) -> &mut RingBuffer {
		&mut self.buffer
	}

	/// Get the number of successes we have recorded in HalfOpen state
	pub fn get_trial_success(&self) -> usize {
		self.trial_success
	}

	/// Get our [Settings]
	pub fn get_settings(&self) -> &Settings {
		&self.settings
	}

	/// Get the error rate calculated for the ring buffer thus far
	pub fn get_error_rate(&self) -> f32 {
		self.buffer.get_error_rate(self.settings.min_eval_size)
	}

	/// Get the elapsed time of our current phase
	pub fn get_elapsed_time(&self, buffer_span_duration: Duration, now: Instant) -> Duration {
		let elapsed = now.duration_since(self.start_time);
		let remainder_ns = elapsed.as_nanos() % buffer_span_duration.as_nanos();
		Duration::from_nanos(remainder_ns as u64)
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
		assert_eq!(CircuitBreaker::new(Settings::default()).buffer.get_size(), 5);
		assert_eq!(CircuitBreaker::new(Settings::default()).settings, Settings::default());
		assert_eq!(
			CircuitBreaker::new(Settings {
				buffer_size: 10,
				..Settings::default()
			})
			.buffer
			.get_size(),
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
	fn advance_buffer_for_time_test() {
		let buffer_span_duration = Duration::from_secs(1);
		let last_record = Instant::now();
		let mut cb = CircuitBreaker {
			buffer: RingBuffer::new(3),
			state: State::Closed,
			last_record,
			start_time: Instant::now(),
			trial_success: 0,
			settings: Settings {
				buffer_span_duration,
				..Settings::default()
			},
		};

		assert_eq!(
			cb.get_buffer().get_node_info(0),
			NodeInfo {
				failure_count: 0,
				success_count: 0,
			}
		);
		assert_eq!(
			cb.get_buffer().get_node_info(1),
			NodeInfo {
				failure_count: 0,
				success_count: 0,
			}
		);
		assert_eq!(
			cb.get_buffer().get_node_info(2),
			NodeInfo {
				failure_count: 0,
				success_count: 0,
			}
		);
		assert_eq!(cb.get_buffer().get_cursor(), 0);
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		assert_eq!(
			cb.get_buffer().get_node_info(0),
			NodeInfo {
				failure_count: 0,
				success_count: 2,
			}
		);
		assert_eq!(
			cb.get_buffer().get_node_info(1),
			NodeInfo {
				failure_count: 0,
				success_count: 0,
			}
		);
		assert_eq!(
			cb.get_buffer().get_node_info(2),
			NodeInfo {
				failure_count: 0,
				success_count: 0,
			}
		);

		cb.advance_buffer_for_time(last_record);
		assert_eq!(cb.get_buffer().get_cursor(), 0);
		assert_eq!(
			cb.get_buffer().get_node_info(0),
			NodeInfo {
				failure_count: 0,
				success_count: 2,
			}
		);
		assert_eq!(
			cb.get_buffer().get_node_info(1),
			NodeInfo {
				failure_count: 0,
				success_count: 0,
			}
		);
		assert_eq!(
			cb.get_buffer().get_node_info(2),
			NodeInfo {
				failure_count: 0,
				success_count: 0,
			}
		);

		cb.advance_buffer_for_time(last_record + buffer_span_duration);
		assert_eq!(cb.get_buffer().get_cursor(), 1);
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Err(""));
		cb.record::<(), &str>(Err(""));
		cb.record::<(), &str>(Ok(()));
		assert_eq!(
			cb.get_buffer().get_node_info(0),
			NodeInfo {
				failure_count: 0,
				success_count: 2,
			}
		);
		assert_eq!(
			cb.get_buffer().get_node_info(1),
			NodeInfo {
				failure_count: 2,
				success_count: 2,
			}
		);
		assert_eq!(
			cb.get_buffer().get_node_info(2),
			NodeInfo {
				failure_count: 0,
				success_count: 0,
			}
		);

		cb.advance_buffer_for_time(last_record + buffer_span_duration + buffer_span_duration);
		assert_eq!(cb.get_buffer().get_cursor(), 2);
		cb.record::<(), &str>(Err(""));
		cb.record::<(), &str>(Ok(()));
		assert_eq!(
			cb.get_buffer().get_node_info(0),
			NodeInfo {
				failure_count: 0,
				success_count: 2,
			}
		);
		assert_eq!(
			cb.get_buffer().get_node_info(1),
			NodeInfo {
				failure_count: 2,
				success_count: 2,
			}
		);
		assert_eq!(
			cb.get_buffer().get_node_info(2),
			NodeInfo {
				failure_count: 1,
				success_count: 1,
			}
		);

		cb.advance_buffer_for_time(
			last_record
				+ buffer_span_duration
				+ buffer_span_duration
				+ buffer_span_duration
				+ buffer_span_duration
				+ buffer_span_duration,
		);
		assert_eq!(cb.get_buffer().get_cursor(), 2);
		assert_eq!(
			cb.get_buffer().get_node_info(0),
			NodeInfo {
				failure_count: 0,
				success_count: 0,
			}
		);
		assert_eq!(
			cb.get_buffer().get_node_info(1),
			NodeInfo {
				failure_count: 0,
				success_count: 0,
			}
		);
		assert_eq!(
			cb.get_buffer().get_node_info(2),
			NodeInfo {
				failure_count: 0,
				success_count: 0,
			}
		);
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Err(""));
		cb.record::<(), &str>(Err(""));
		assert_eq!(
			cb.get_buffer().get_node_info(0),
			NodeInfo {
				failure_count: 0,
				success_count: 0,
			}
		);
		assert_eq!(
			cb.get_buffer().get_node_info(1),
			NodeInfo {
				failure_count: 0,
				success_count: 0,
			}
		);
		assert_eq!(
			cb.get_buffer().get_node_info(2),
			NodeInfo {
				failure_count: 2,
				success_count: 1,
			}
		);
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
		let buffer_span_duration = Duration::from_secs(1);
		let mut cb = CircuitBreaker::new(Settings {
			buffer_span_duration,
			..Settings::default()
		});

		cb.advance_buffer_for_time(Instant::now());
		assert_eq!(cb.get_buffer().get_cursor(), 0);
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		std::thread::sleep(buffer_span_duration);
		cb.advance_buffer_for_time(Instant::now());
		assert_eq!(cb.get_buffer().get_cursor(), 1);
		assert_eq!(cb.buffer.get_node_info(0).success_count, 3);
		assert_eq!(cb.buffer.get_node_info(1).success_count, 0);
		assert_eq!(cb.buffer.get_node_info(2).success_count, 0);
		assert_eq!(cb.buffer.get_node_info(3).success_count, 0);
		assert_eq!(cb.buffer.get_node_info(4).success_count, 0);
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		std::thread::sleep(buffer_span_duration);
		cb.advance_buffer_for_time(Instant::now());
		assert_eq!(cb.get_buffer().get_cursor(), 2);
		assert_eq!(cb.buffer.get_node_info(0).success_count, 3);
		assert_eq!(cb.buffer.get_node_info(1).success_count, 3);
		assert_eq!(cb.buffer.get_node_info(2).success_count, 0);
		assert_eq!(cb.buffer.get_node_info(3).success_count, 0);
		assert_eq!(cb.buffer.get_node_info(4).success_count, 0);
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		std::thread::sleep(buffer_span_duration);
		cb.advance_buffer_for_time(Instant::now());
		assert_eq!(cb.get_buffer().get_cursor(), 3);
		assert_eq!(cb.buffer.get_node_info(0).success_count, 3);
		assert_eq!(cb.buffer.get_node_info(1).success_count, 3);
		assert_eq!(cb.buffer.get_node_info(2).success_count, 3);
		assert_eq!(cb.buffer.get_node_info(3).success_count, 0);
		assert_eq!(cb.buffer.get_node_info(4).success_count, 0);
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		std::thread::sleep(buffer_span_duration);
		cb.advance_buffer_for_time(Instant::now());
		assert_eq!(cb.get_buffer().get_cursor(), 4);
		assert_eq!(cb.buffer.get_node_info(0).success_count, 3);
		assert_eq!(cb.buffer.get_node_info(1).success_count, 3);
		assert_eq!(cb.buffer.get_node_info(2).success_count, 3);
		assert_eq!(cb.buffer.get_node_info(3).success_count, 3);
		assert_eq!(cb.buffer.get_node_info(4).success_count, 0);
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		std::thread::sleep(buffer_span_duration);
		cb.advance_buffer_for_time(Instant::now());
		assert_eq!(cb.get_buffer().get_cursor(), 0);
		assert_eq!(cb.buffer.get_node_info(0).success_count, 0);
		assert_eq!(cb.buffer.get_node_info(1).success_count, 3);
		assert_eq!(cb.buffer.get_node_info(2).success_count, 3);
		assert_eq!(cb.buffer.get_node_info(3).success_count, 3);
		assert_eq!(cb.buffer.get_node_info(4).success_count, 3);
		cb.record::<(), &str>(Ok(()));

		// We skip 3 nodes ahead
		std::thread::sleep(buffer_span_duration + buffer_span_duration + buffer_span_duration);
		cb.evaluate_state();

		assert_eq!(cb.buffer.get_node_info(0).success_count, 1);
		assert_eq!(cb.buffer.get_node_info(1).success_count, 0); // skipped
		assert_eq!(cb.buffer.get_node_info(2).success_count, 0); // skipped
		assert_eq!(cb.buffer.get_node_info(3).success_count, 0); // current
		assert_eq!(cb.buffer.get_node_info(4).success_count, 3);
		cb.advance_buffer_for_time(Instant::now());
		assert_eq!(cb.get_buffer().get_cursor(), 3);
	}

	#[test]
	fn evaluate_state_test() {
		// Open state within the retry_timeout time
		let retry_timeout = Duration::from_secs(1);
		let mut cb = CircuitBreaker {
			buffer: RingBuffer::new(5),
			state: State::Open(Instant::now()),
			last_record: Instant::now(),
			start_time: Instant::now(),
			trial_success: 0,
			settings: Settings {
				retry_timeout,
				..Settings::default()
			},
		};
		cb.evaluate_state();
		assert!(matches!(cb.get_state(), State::Open(_)));

		// Open state after the retry_timeout time
		let retry_timeout = Duration::from_secs(1);
		let mut cb = CircuitBreaker {
			buffer: RingBuffer::new(5),
			state: State::Open(Instant::now() - retry_timeout),
			last_record: Instant::now(),
			start_time: Instant::now(),
			trial_success: 0,
			settings: Settings {
				retry_timeout,
				..Settings::default()
			},
		};
		cb.evaluate_state();
		assert_eq!(cb.get_state(), State::HalfOpen);

		// Closed state within the margin of error
		let buffer_span_duration = Duration::from_secs(1);
		let mut cb = CircuitBreaker {
			buffer: RingBuffer::new(5),
			state: State::Closed,
			last_record: Instant::now(),
			start_time: Instant::now(),
			trial_success: 0,
			settings: Settings {
				min_eval_size: 4,
				error_threshold: 39.99999,
				buffer_span_duration,
				..Settings::default()
			},
		};
		cb.record::<(), &str>(Err(""));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.advance_buffer_for_time(Instant::now() + buffer_span_duration);
		assert_eq!(cb.get_error_rate(), 20.0);
		cb.evaluate_state();
		assert_eq!(cb.get_state(), State::Closed);

		// Closed state an error larger than error_threshold
		let buffer_span_duration = Duration::from_secs(1);
		let mut cb = CircuitBreaker {
			buffer: RingBuffer::new(5),
			state: State::Closed,
			last_record: Instant::now(),
			start_time: Instant::now(),
			trial_success: 0,
			settings: Settings {
				min_eval_size: 4,
				error_threshold: 39.99999,
				buffer_span_duration,
				..Settings::default()
			},
		};
		cb.record::<(), &str>(Err(""));
		cb.record::<(), &str>(Err(""));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.advance_buffer_for_time(Instant::now() + buffer_span_duration);
		assert_eq!(cb.get_error_rate(), 40.0);
		cb.evaluate_state();
		assert!(matches!(cb.get_state(), State::Open(_)));

		// HalfOpen state with slowly increasing trial_success
		let mut cb = CircuitBreaker {
			buffer: RingBuffer::new(5),
			state: State::HalfOpen,
			last_record: Instant::now(),
			start_time: Instant::now(),
			trial_success: 0,
			settings: Settings {
				trial_success_required: 5,
				..Settings::default()
			},
		};
		cb.evaluate_state();
		assert_eq!(cb.get_state(), State::HalfOpen);
		cb.trial_success = 4;
		cb.evaluate_state();
		assert_eq!(cb.get_state(), State::HalfOpen);
		cb.trial_success += 1;
		cb.evaluate_state();
		assert_eq!(cb.get_state(), State::Closed);
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

	#[test]
	fn get_elapsed_time_test() {
		let timeout = Instant::now();
		let cb = CircuitBreaker {
			start_time: timeout,
			last_record: timeout,
			..CircuitBreaker::default()
		};

		assert_eq!(cb.get_elapsed_time(Duration::from_secs(5), timeout + Duration::from_secs(1)), Duration::from_secs(1));
		assert_eq!(cb.get_elapsed_time(Duration::from_secs(5), timeout + Duration::from_secs(4)), Duration::from_secs(4));
		assert_eq!(cb.get_elapsed_time(Duration::from_secs(5), timeout + Duration::from_secs(5)), Duration::from_secs(0));
		assert_eq!(cb.get_elapsed_time(Duration::from_secs(5), timeout + Duration::from_secs(6)), Duration::from_secs(1));
	}

	#[test]
	fn end_2_end_test() {
		let buffer_span_duration = Duration::from_millis(300);
		let retry_timeout = Duration::from_millis(200);
		let mut cb = CircuitBreaker::new(Settings {
			buffer_span_duration,
			retry_timeout,
			min_eval_size: 5,
			trial_success_required: 3,
			..Settings::default()
		});

		let cursor = cb.get_buffer().get_cursor();
		assert_eq!(cursor, 0);
		assert_eq!(
			cb.get_buffer().get_node_info(cursor),
			NodeInfo {
				success_count: 0,
				failure_count: 0,
			}
		);
		assert_eq!(cb.get_state(), State::Closed);
		assert_eq!(cb.get_error_rate(), 0.0);

		cb.record::<(), &str>(Ok(()));
		let cursor = cb.get_buffer().get_cursor();
		assert_eq!(cursor, 0);
		assert_eq!(
			cb.get_buffer().get_node_info(cursor),
			NodeInfo {
				success_count: 1,
				failure_count: 0,
			}
		);
		assert_eq!(cb.get_state(), State::Closed);
		assert_eq!(cb.get_error_rate(), 0.0);
		std::thread::sleep(buffer_span_duration);

		assert_eq!(cb.get_state(), State::Closed);
		assert_eq!(cb.get_buffer().get_cursor(), 1);
		cb.record::<(), &str>(Err(""));
		cb.record::<(), &str>(Err(""));
		cb.record::<(), &str>(Err(""));
		cb.record::<(), &str>(Err(""));
		cb.record::<(), &str>(Err(""));
		std::thread::sleep(buffer_span_duration);
		let cursor = cb.get_buffer().get_cursor();
		assert_eq!(cursor, 1);
		assert_eq!(
			cb.get_buffer().get_node_info(cursor),
			NodeInfo {
				success_count: 0,
				failure_count: 5,
			}
		);
		assert!(matches!(cb.get_state(), State::Open(_)));
		assert_eq!(cb.get_error_rate(), 83.33);

		cb.record::<(), &str>(Err(""));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Err(""));
		let cursor = cb.get_buffer().get_cursor();
		assert_eq!(cursor, 2);
		assert_eq!(
			cb.get_buffer().get_node_info(cursor),
			NodeInfo {
				success_count: 0,
				failure_count: 0,
			}
		);
		assert!(matches!(cb.get_state(), State::Open(_)));
		assert_eq!(cb.get_error_rate(), 83.33);

		std::thread::sleep(retry_timeout);
		let cursor = cb.get_buffer().get_cursor();
		assert_eq!(cursor, 2);
		assert_eq!(
			cb.get_buffer().get_node_info(cursor),
			NodeInfo {
				success_count: 0,
				failure_count: 0,
			}
		);
		assert_eq!(cb.get_state(), State::HalfOpen);
		assert_eq!(cb.get_error_rate(), 83.33);

		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));

		let cursor = cb.get_buffer().get_cursor();
		assert_eq!(cursor, 2);
		assert_eq!(
			cb.get_buffer().get_node_info(cursor),
			NodeInfo {
				success_count: 0,
				failure_count: 0,
			}
		);
		assert_eq!(cb.get_state(), State::HalfOpen);
		assert_eq!(cb.get_error_rate(), 83.33);

		cb.record::<(), &str>(Err(""));

		let cursor = cb.get_buffer().get_cursor();
		assert_eq!(cursor, 2);
		assert_eq!(
			cb.get_buffer().get_node_info(cursor),
			NodeInfo {
				success_count: 0,
				failure_count: 0,
			}
		);
		assert!(matches!(cb.get_state(), State::Open(_)));
		assert_eq!(cb.get_error_rate(), 83.33);

		std::thread::sleep(retry_timeout);

		let cursor = cb.get_buffer().get_cursor();
		assert_eq!(cursor, 2);
		assert_eq!(
			cb.get_buffer().get_node_info(cursor),
			NodeInfo {
				success_count: 0,
				failure_count: 0,
			}
		);
		assert_eq!(cb.get_state(), State::HalfOpen);
		assert_eq!(cb.get_error_rate(), 83.33);

		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));

		let cursor = cb.get_buffer().get_cursor();
		assert_eq!(cursor, 0);
		assert_eq!(
			cb.get_buffer().get_node_info(cursor),
			NodeInfo {
				success_count: 0,
				failure_count: 0,
			}
		);
		assert_eq!(cb.get_state(), State::Closed);
		assert_eq!(cb.get_error_rate(), 0.0);

		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Ok(()));
		cb.record::<(), &str>(Err(""));

		let cursor = cb.get_buffer().get_cursor();
		assert_eq!(cursor, 0);
		assert_eq!(
			cb.get_buffer().get_node_info(cursor),
			NodeInfo {
				success_count: 4,
				failure_count: 1,
			}
		);
		assert_eq!(cb.get_state(), State::Closed);
		assert_eq!(cb.get_error_rate(), 0.0);
	}
}
