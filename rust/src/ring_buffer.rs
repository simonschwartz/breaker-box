use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Node {
	failure_count: usize,
	success_count: usize,
}

impl Node {
	pub fn new() -> Self {
		Self {
			failure_count: 0,
			success_count: 0,
		}
	}

	pub fn reset(&mut self) {
		self.failure_count = 0;
		self.success_count = 0;
	}
}

impl Default for Node {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeInfo {
	pub failure_count: usize,
	pub success_count: usize,
}

#[derive(Debug, PartialEq)]
pub struct RingBuffer {
	cursor: usize,
	nodes: Vec<Node>,
	start_time: Instant,
	last_record: Instant,
}

impl RingBuffer {
	pub fn new(elements: usize) -> Self {
		Self {
			cursor: 0,
			nodes: vec![Node::new(); elements],
			start_time: Instant::now(),
			last_record: Instant::now(),
		}
	}

	pub fn reset_start_time(&mut self) {
		self.start_time = Instant::now();
		self.last_record = Instant::now();
		self.cursor = self.cursor + 1 % self.get_buffer_size();
	}

	pub fn get_buffer_size(&self) -> usize {
		self.nodes.len()
	}

	pub fn get_cursor(&mut self, buffer_span_duration: Duration, now: Instant) -> usize {
		let elapsed = now.duration_since(self.start_time);
		let spans_elapsed = elapsed.as_nanos() / buffer_span_duration.as_nanos();
		let index = (spans_elapsed + self.cursor as u128) % (self.get_buffer_size() as u128);
		let new_cursor = index as usize;
		let buffer_size = self.get_buffer_size();
		let cursor_advancement = (new_cursor + 1 + buffer_size - self.cursor) % buffer_size;
		for step in 2..cursor_advancement {
			let skip_idx = (self.cursor + step) % buffer_size;
			self.nodes[skip_idx].reset();
		}
		self.cursor = new_cursor;
		self.cursor
	}

	pub fn add_failure(&mut self, buffer_span_duration: Duration) {
		let index = self.get_cursor(buffer_span_duration, Instant::now());
		self.nodes[index].failure_count += 1;
		self.last_record = Instant::now();
	}

	pub fn add_success(&mut self, buffer_span_duration: Duration) {
		let index = self.get_cursor(buffer_span_duration, Instant::now());
		self.nodes[index].success_count += 1;
		self.last_record = Instant::now();
	}

	pub fn get_elapsed_time(&self, buffer_span_duration: Duration, now: Instant) -> Duration {
		let elapsed = now.duration_since(self.start_time);
		let remainder_ns = elapsed.as_nanos() % buffer_span_duration.as_nanos();
		Duration::from_nanos(remainder_ns as u64)
	}

	pub fn get_error_rate(&self, min_eval_size: usize) -> f32 {
		let mut failures = 0;
		let mut successes = 0;

		for (i, node) in self.nodes.iter().enumerate() {
			if i == self.cursor {
				continue;
			}

			if node.failure_count + node.success_count != 0 {
				failures += node.failure_count;
				successes += node.success_count;
			}
		}

		if failures + successes < min_eval_size || failures + successes == 0 {
			0.0
		} else {
			((failures as f32 / (failures + successes) as f32) * 10_000.0).round() / 100.0
		}
	}

	pub fn get_node_info(&self, index: usize) -> NodeInfo {
		NodeInfo {
			failure_count: self.nodes[index].failure_count,
			success_count: self.nodes[index].success_count,
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn new_test() {
		assert_eq!(RingBuffer::new(1).nodes.len(), 1);
		assert_eq!(RingBuffer::new(1).nodes[0].failure_count, 0);
		assert_eq!(RingBuffer::new(1).nodes[0].success_count, 0);
		assert_eq!(RingBuffer::new(5).nodes.len(), 5);
		assert_eq!(RingBuffer::new(5).nodes[4].failure_count, 0);
		assert_eq!(RingBuffer::new(5).nodes[4].success_count, 0);
		assert_eq!(RingBuffer::new(100).nodes.len(), 100);
	}

	#[test]
	fn get_buffer_size_test() {
		assert_eq!(RingBuffer::new(1).get_buffer_size(), 1);
		assert_eq!(RingBuffer::new(5).get_buffer_size(), 5);
		assert_eq!(RingBuffer::new(100).get_buffer_size(), 100);
	}

	#[test]
	fn get_cursor_test() {
		let start_time = Instant::now();
		let mut rb = RingBuffer {
			cursor: 0,
			nodes: vec![Node::new(); 10],
			start_time,
			last_record: start_time,
		};

		assert_eq!(rb.get_cursor(Duration::from_secs(1), start_time + Duration::from_secs(0)), 0);
		assert_eq!(rb.get_cursor(Duration::from_secs(1), start_time + Duration::from_millis(999)), 0);
		assert_eq!(rb.get_cursor(Duration::from_secs(1), start_time + Duration::from_secs(1)), 1);
		rb.cursor = 0;
		rb.nodes[0].failure_count = 5;
		rb.nodes[0].success_count = 7;
		rb.nodes[1].failure_count = 666;
		rb.nodes[1].success_count = 667;
		rb.nodes[2].failure_count = 42;
		rb.nodes[2].success_count = 666;
		rb.nodes[3].failure_count = 99;
		rb.nodes[3].success_count = 5;
		rb.nodes[6].failure_count = 24;
		rb.nodes[6].success_count = 7;
		rb.nodes[7].failure_count = 666;
		rb.nodes[7].success_count = 42;
		assert_eq!(rb.get_cursor(Duration::from_secs(1), start_time + Duration::from_secs(6)), 6);
		assert_eq!(rb.nodes[0].failure_count, 5);
		assert_eq!(rb.nodes[0].success_count, 7);
		assert_eq!(rb.nodes[1].failure_count, 666);
		assert_eq!(rb.nodes[1].success_count, 667);
		assert_eq!(
			rb.nodes[2].failure_count
				+ rb.nodes[2].success_count
				+ rb.nodes[3].failure_count
				+ rb.nodes[3].success_count
				+ rb.nodes[6].failure_count
				+ rb.nodes[6].success_count,
			0
		);
		assert_eq!(rb.nodes[7].failure_count, 666);
		assert_eq!(rb.nodes[7].success_count, 42);
		rb.cursor = 0;
		assert_eq!(rb.get_cursor(Duration::from_secs(1), start_time + Duration::from_secs(9)), 9);
		rb.cursor = 0;
		assert_eq!(rb.get_cursor(Duration::from_secs(1), start_time + Duration::from_secs(10)), 0);
		rb.cursor = 0;
		assert_eq!(rb.get_cursor(Duration::from_secs(1), start_time + Duration::from_secs(99)), 9);
		rb.cursor = 0;
		assert_eq!(rb.get_cursor(Duration::from_secs(1), start_time + Duration::from_secs(100)), 0);
		assert_eq!(rb.get_cursor(Duration::from_secs(1), start_time + Duration::from_secs(9)), 9);
		assert_eq!(rb.get_cursor(Duration::from_secs(1), start_time + Duration::from_secs(1)), 0);
		assert_eq!(rb.get_cursor(Duration::from_secs(1), start_time + Duration::from_secs(4)), 4);
		assert_eq!(rb.get_cursor(Duration::from_secs(1), start_time + Duration::from_secs(100)), 4);
	}

	#[test]
	fn add_failure_success_test() {
		let mut buffer = RingBuffer::new(1);

		assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 0);
		assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);
		buffer.add_failure(Duration::from_secs(10));
		assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 1);
		assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);
		buffer.add_success(Duration::from_secs(10));
		assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 1);
		assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 1);
		buffer.add_success(Duration::from_secs(10));
		assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 1);
		assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 2);
		buffer.add_failure(Duration::from_secs(10));
		assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 2);
		assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 2);
	}

	#[test]
	fn next_add_failure_success_test() {
		let mut buffer = RingBuffer::new(3);
		assert_eq!(buffer.cursor, 0);
		assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 0);
		assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);
		buffer.add_failure(Duration::from_secs(10));
		assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 1);
		assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);
		buffer.add_failure(Duration::from_secs(10));
		assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 2);
		assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);
		buffer.cursor += 1;
		assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 0);
		assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);
		buffer.add_success(Duration::from_secs(10));
		assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 0);
		assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 1);
		buffer.add_success(Duration::from_secs(10));
		assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 0);
		assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 2);
		buffer.cursor += 1;
		assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 0);
		assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);
		buffer.add_success(Duration::from_secs(10));
		assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 0);
		assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 1);
		buffer.add_failure(Duration::from_secs(10));
		assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 1);
		assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 1);
	}

	#[test]
	fn get_error_rate_test() {
		let buffer = RingBuffer {
			cursor: 0,
			nodes: vec![
				Node {
					failure_count: 50,
					success_count: 50,
				},
				Node {
					failure_count: 0,
					success_count: 0,
				},
			],
			start_time: Instant::now(),
			last_record: Instant::now(),
		};
		assert_eq!(buffer.get_error_rate(10), 0.0); // cursor on first node

		let buffer = RingBuffer {
			cursor: 1,
			nodes: vec![
				Node {
					failure_count: 50,
					success_count: 50,
				},
				Node {
					failure_count: 0,
					success_count: 0,
				},
			],
			start_time: Instant::now(),
			last_record: Instant::now(),
		};
		assert_eq!(buffer.get_error_rate(10), 50.0); // 50 of 100 = 50%

		let buffer = RingBuffer {
			cursor: 0,
			nodes: vec![
				Node {
					failure_count: 0,
					success_count: 0,
				},
				Node {
					failure_count: 50,
					success_count: 50,
				},
				Node {
					failure_count: 10,
					success_count: 90,
				},
			],
			start_time: Instant::now(),
			last_record: Instant::now(),
		};
		assert_eq!(buffer.get_error_rate(10), 30.0); // 60 of 200 = 30%

		let buffer = RingBuffer {
			cursor: 0,
			nodes: vec![
				Node {
					failure_count: 0,
					success_count: 0,
				},
				Node {
					failure_count: 5,
					success_count: 5,
				},
				Node {
					failure_count: 1,
					success_count: 9,
				},
			],
			start_time: Instant::now(),
			last_record: Instant::now(),
		};
		assert_eq!(buffer.get_error_rate(100), 0.0); // 6 of 20 = 30% but less than min_eval_size
	}

	#[test]
	fn get_node_info_test() {
		let buffer = RingBuffer {
			cursor: 0,
			nodes: vec![
				Node {
					failure_count: 42,
					success_count: 666,
				},
				Node {
					failure_count: 0,
					success_count: 42,
				},
				Node {
					failure_count: 256,
					success_count: 0,
				},
			],
			start_time: Instant::now(),
			last_record: Instant::now(),
		};

		assert_eq!(
			buffer.get_node_info(0),
			NodeInfo {
				failure_count: 42,
				success_count: 666,
			}
		);
		assert_eq!(
			buffer.get_node_info(1),
			NodeInfo {
				failure_count: 0,
				success_count: 42,
			}
		);
		assert_eq!(
			buffer.get_node_info(2),
			NodeInfo {
				failure_count: 256,
				success_count: 0,
			}
		);
	}
}
