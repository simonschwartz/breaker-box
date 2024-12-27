use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Node {
	created: Instant,
	failure_count: usize,
	success_count: usize,
}

impl Node {
	pub fn new() -> Self {
		Self {
			created: Instant::now(),
			failure_count: 0,
			success_count: 0,
		}
	}

	pub fn reset(&mut self) {
		self.created = Instant::now();
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
}

impl RingBuffer {
	pub fn new(elements: usize) -> Self {
		Self {
			cursor: 0,
			nodes: vec![Node::new(); elements],
		}
	}

	pub fn get_length(&self) -> usize {
		self.nodes.len()
	}

	pub fn get_cursor(&self) -> usize {
		self.cursor
	}

	pub fn add_failure(&mut self) {
		self.nodes[self.cursor].failure_count += 1;
	}

	pub fn add_success(&mut self) {
		self.nodes[self.cursor].success_count += 1;
	}

	pub fn has_exired(&self, timeout: Duration) -> bool {
		self.nodes[self.cursor].created.elapsed() >= timeout
	}

	pub fn get_elapsed_time(&self) -> Duration {
		self.nodes[self.cursor].created.elapsed()
	}

	pub fn next(&mut self) {
		if self.cursor == self.nodes.len() - 1 {
			self.cursor = 0;
		} else {
			self.cursor += 1;
		}
		self.nodes[self.cursor].reset();
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
	fn get_length_test() {
		assert_eq!(RingBuffer::new(1).get_length(), 1);
		assert_eq!(RingBuffer::new(5).get_length(), 5);
		assert_eq!(RingBuffer::new(100).get_length(), 100);
	}

	#[test]
	fn get_cursor_test() {
		let mut buffer = RingBuffer::new(3);
		assert_eq!(buffer.get_cursor(), 0);
		buffer.next();
		assert_eq!(buffer.get_cursor(), 1);
		buffer.next();
		assert_eq!(buffer.get_cursor(), 2);
		buffer.next();
		assert_eq!(buffer.get_cursor(), 0);
		buffer.next();
		assert_eq!(buffer.get_cursor(), 1);
	}

	#[test]
	fn add_failure_success_test() {
		let mut buffer = RingBuffer::new(1);

		assert_eq!(buffer.get_node_info(buffer.get_cursor()).failure_count, 0);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).success_count, 0);
		buffer.add_failure();
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).failure_count, 1);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).success_count, 0);
		buffer.add_success();
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).failure_count, 1);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).success_count, 1);
		buffer.add_success();
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).failure_count, 1);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).success_count, 2);
		buffer.add_failure();
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).failure_count, 2);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).success_count, 2);
	}

	#[test]
	fn has_expired_test() {
		let buffer = RingBuffer {
			cursor: 0,
			nodes: vec![Node {
				created: Instant::now().checked_add(Duration::from_secs(10)).unwrap(),
				failure_count: 0,
				success_count: 0,
			}],
		};
		assert_eq!(buffer.has_exired(Duration::from_secs(2)), false);

		let buffer = RingBuffer {
			cursor: 0,
			nodes: vec![Node {
				created: Instant::now().checked_sub(Duration::from_secs(100)).unwrap(),
				failure_count: 0,
				success_count: 0,
			}],
		};
		assert_eq!(buffer.has_exired(Duration::from_secs(100)), true);

		let buffer = RingBuffer {
			cursor: 0,
			nodes: vec![Node {
				created: Instant::now(),
				failure_count: 0,
				success_count: 0,
			}],
		};
		assert_eq!(buffer.has_exired(Duration::from_secs(10)), false);
	}

	#[test]
	fn get_elapsed_test() {
		let timeout = Instant::now().checked_sub(Duration::from_secs(100)).unwrap();
		let buffer = RingBuffer {
			cursor: 0,
			nodes: vec![Node {
				created: timeout,
				failure_count: 0,
				success_count: 0,
			}],
		};
		assert_eq!(buffer.get_elapsed_time().as_secs(), timeout.elapsed().as_secs());
	}

	#[test]
	fn next_test() {
		let mut buffer = RingBuffer::new(3);
		assert_eq!(buffer.cursor, 0);
		buffer.next();
		assert_eq!(buffer.cursor, 1);
		buffer.next();
		assert_eq!(buffer.cursor, 2);
		buffer.next();
		assert_eq!(buffer.cursor, 0);
		buffer.next();
		assert_eq!(buffer.cursor, 1);
		buffer.next();
		assert_eq!(buffer.cursor, 2);
		buffer.next();
		assert_eq!(buffer.cursor, 0);

		let mut buffer = RingBuffer::new(1);
		assert_eq!(buffer.cursor, 0);
		buffer.next();
		assert_eq!(buffer.cursor, 0);
		buffer.next();
		assert_eq!(buffer.cursor, 0);
		buffer.next();
		assert_eq!(buffer.cursor, 0);
	}

	#[test]
	fn next_add_failure_success_test() {
		let mut buffer = RingBuffer::new(3);
		assert_eq!(buffer.cursor, 0);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).failure_count, 0);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).success_count, 0);
		buffer.add_failure();
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).failure_count, 1);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).success_count, 0);
		buffer.add_failure();
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).failure_count, 2);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).success_count, 0);
		buffer.next();
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).failure_count, 0);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).success_count, 0);
		buffer.add_success();
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).failure_count, 0);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).success_count, 1);
		buffer.add_success();
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).failure_count, 0);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).success_count, 2);
		buffer.next();
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).failure_count, 0);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).success_count, 0);
		buffer.add_success();
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).failure_count, 0);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).success_count, 1);
		buffer.add_failure();
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).failure_count, 1);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).success_count, 1);
		buffer.next();
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).failure_count, 0);
		assert_eq!(buffer.get_node_info(buffer.get_cursor()).success_count, 0);
	}

	#[test]
	fn get_error_rate_test() {
		let buffer = RingBuffer {
			cursor: 0,
			nodes: vec![
				Node {
					created: Instant::now(),
					failure_count: 50,
					success_count: 50,
				},
				Node {
					created: Instant::now(),
					failure_count: 0,
					success_count: 0,
				},
			],
		};
		assert_eq!(buffer.get_error_rate(10), 0.0); // cursor on first node

		let buffer = RingBuffer {
			cursor: 1,
			nodes: vec![
				Node {
					created: Instant::now(),
					failure_count: 50,
					success_count: 50,
				},
				Node {
					created: Instant::now(),
					failure_count: 0,
					success_count: 0,
				},
			],
		};
		assert_eq!(buffer.get_error_rate(10), 50.0); // 50 of 100 = 50%

		let buffer = RingBuffer {
			cursor: 0,
			nodes: vec![
				Node {
					created: Instant::now(),
					failure_count: 0,
					success_count: 0,
				},
				Node {
					created: Instant::now(),
					failure_count: 50,
					success_count: 50,
				},
				Node {
					created: Instant::now(),
					failure_count: 10,
					success_count: 90,
				},
			],
		};
		assert_eq!(buffer.get_error_rate(10), 30.0); // 60 of 200 = 30%

		let buffer = RingBuffer {
			cursor: 0,
			nodes: vec![
				Node {
					created: Instant::now(),
					failure_count: 0,
					success_count: 0,
				},
				Node {
					created: Instant::now(),
					failure_count: 5,
					success_count: 5,
				},
				Node {
					created: Instant::now(),
					failure_count: 1,
					success_count: 9,
				},
			],
		};
		assert_eq!(buffer.get_error_rate(100), 0.0); // 6 of 20 = 30% but less than min_eval_size
	}

	#[test]
	fn get_node_info_test() {
		let buffer = RingBuffer {
			cursor: 0,
			nodes: vec![
				Node {
					created: Instant::now(),
					failure_count: 42,
					success_count: 666,
				},
				Node {
					created: Instant::now(),
					failure_count: 0,
					success_count: 42,
				},
				Node {
					created: Instant::now(),
					failure_count: 256,
					success_count: 0,
				},
			],
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
