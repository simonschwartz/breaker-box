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
}

impl RingBuffer {
	/// Create a new ring buffer with `elements` [Node]
	pub fn new(elements: usize) -> Self {
		Self {
			cursor: 0,
			nodes: vec![Node::new(); elements],
		}
	}

	/// Returns the size of the buffer
	pub fn get_size(&self) -> usize {
		self.nodes.len()
	}

	/// Returns the current cursor
	pub fn get_cursor(&self) -> usize {
		self.cursor
	}

	/// Move the cursor forward by `steps` positions (modulo buffer size),
	/// resetting any nodes we skip along the way
	pub fn advance(&mut self, steps: usize) {
		if self.nodes.is_empty() {
			return;
		}

		let size = self.get_size();

		let start = self.cursor + 1;
		let end = self.cursor + steps + 1;
		if steps >= size {
			for node in &mut self.nodes {
				node.reset();
			}
		} else {
			for idx in start..end {
				let skip_idx = idx % size;
				self.nodes[skip_idx].reset();
			}
		}

		self.cursor = (self.cursor + steps) % size;
		self.nodes[self.cursor].reset();
	}

	/// Increments the failure count at the current cursor
	pub fn add_failure(&mut self) {
		self.nodes[self.cursor].failure_count += 1;
	}

	/// Increments the success count at the current cursor
	pub fn add_success(&mut self) {
		self.nodes[self.cursor].success_count += 1;
	}

	/// Retrieve info for a specific node
	pub fn get_node_info(&self, index: usize) -> NodeInfo {
		if index > self.nodes.len() {
			panic!("Index out of bounds");
		}

		NodeInfo {
			failure_count: self.nodes[index].failure_count,
			success_count: self.nodes[index].success_count,
		}
	}

	/// Returns the error rate as a percentage (0.0 to 100.0)
	/// If `failures+successes` < `min_eval_size`, returns 0.0
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
	fn get_size_test() {
		assert_eq!(RingBuffer::new(1).get_size(), 1);
		assert_eq!(RingBuffer::new(5).get_size(), 5);
		assert_eq!(RingBuffer::new(100).get_size(), 100);
	}

	// #[test]
	// fn get_cursor_timed_test() {
	// 	let mut rb = RingBuffer {
	// 		cursor: 0,
	// 		nodes: vec![Node::new(); 5],
	// 		start_time: Instant::now(),
	// 		last_record: Instant::now(),
	// 	};
	// 	let buffer_span_duration = Duration::from_secs(1);

	// 	rb.add_success(buffer_span_duration, Instant::now());
	// 	rb.add_success(buffer_span_duration, Instant::now());
	// 	rb.add_success(buffer_span_duration, Instant::now());
	// 	std::thread::sleep(Duration::from_secs(1));
	// 	rb.add_success(buffer_span_duration, Instant::now());
	// 	rb.add_success(buffer_span_duration, Instant::now());
	// 	rb.add_success(buffer_span_duration, Instant::now());
	// 	std::thread::sleep(Duration::from_secs(1));
	// 	rb.add_success(buffer_span_duration, Instant::now());
	// 	rb.add_success(buffer_span_duration, Instant::now());
	// 	rb.add_success(buffer_span_duration, Instant::now());
	// 	std::thread::sleep(Duration::from_secs(1));
	// 	rb.add_success(buffer_span_duration, Instant::now());
	// 	rb.add_success(buffer_span_duration, Instant::now());
	// 	rb.add_success(buffer_span_duration, Instant::now());
	// 	std::thread::sleep(Duration::from_secs(1));
	// 	rb.add_success(buffer_span_duration, Instant::now());
	// 	rb.add_success(buffer_span_duration, Instant::now());
	// 	rb.add_success(buffer_span_duration, Instant::now());

	// 	// We skip 3 nodes ahead
	// 	std::thread::sleep(Duration::from_secs(2));
	// 	assert_eq!(rb.get_cursor(buffer_span_duration, Instant::now()), 1);

	// 	assert_eq!(rb.nodes[0].success_count, 0); // skipped
	// 	assert_eq!(rb.nodes[1].success_count, 0); // current
	// 	assert_eq!(rb.nodes[2].success_count, 3);
	// 	assert_eq!(rb.nodes[3].success_count, 3);
	// 	assert_eq!(rb.nodes[4].success_count, 3);
	// }

	// #[test]
	// fn get_cursor_test() {
	// 	let start_time = Instant::now();
	// 	let mut rb = RingBuffer {
	// 		cursor: 0,
	// 		nodes: vec![Node::new(); 10],
	// 		start_time,
	// 		last_record: start_time,
	// 	};
	// 	let buffer_span_duration = Duration::from_secs(1);

	// 	assert_eq!(rb.get_cursor(buffer_span_duration, start_time + Duration::from_secs(0)), 0);
	// 	assert_eq!(rb.get_cursor(buffer_span_duration, start_time + Duration::from_millis(999)), 0);

	// 	// we are pretending something has been recorded into the second node
	// 	// Node 1 => Node 1
	// 	rb.last_record = start_time + Duration::from_millis(1000);
	// 	rb.nodes[0].failure_count = 5;
	// 	rb.nodes[0].success_count = 7;
	// 	rb.nodes[1].failure_count = 666;
	// 	rb.nodes[1].success_count = 667;
	// 	rb.nodes[2].failure_count = 42;
	// 	rb.nodes[2].success_count = 666;
	// 	rb.nodes[3].failure_count = 99;
	// 	rb.nodes[3].success_count = 5;
	// 	rb.nodes[5].failure_count = 999;
	// 	rb.nodes[5].success_count = 999;
	// 	rb.nodes[4].failure_count = 999;
	// 	rb.nodes[4].success_count = 999;
	// 	rb.nodes[6].failure_count = 24;
	// 	rb.nodes[6].success_count = 7;
	// 	rb.nodes[7].failure_count = 666;
	// 	rb.nodes[7].success_count = 42;
	// 	assert_eq!(rb.get_cursor(buffer_span_duration, start_time + Duration::from_millis(1000)), 1);
	// 	assert_eq!(rb.get_cursor(buffer_span_duration, start_time + Duration::from_millis(1000)), 1); // duplicate to make sure we are deterministic
	// 	assert_eq!(rb.get_cursor(buffer_span_duration, start_time + Duration::from_millis(1001)), 1);
	// 	assert_eq!(rb.get_cursor(buffer_span_duration, start_time + Duration::from_millis(1002)), 1);
	// 	assert_eq!(rb.get_cursor(buffer_span_duration, start_time + Duration::from_millis(1003)), 1);
	// 	assert_eq!(rb.nodes[0].failure_count, 5);
	// 	assert_eq!(rb.nodes[0].success_count, 7);
	// 	assert_eq!(rb.nodes[1].failure_count, 0); // node 1 was reset as we just moved here
	// 	assert_eq!(rb.nodes[1].success_count, 0);

	// 	// now we skip to node 6 and see if only the skipped nodes were reset
	// 	// Node 1 => Node 6
	// 	rb.nodes[1].failure_count = 666;
	// 	rb.nodes[1].success_count = 667;
	// 	assert_eq!(rb.get_cursor(buffer_span_duration, start_time + Duration::from_secs(6)), 6);
	// 	assert_eq!(rb.nodes[0].failure_count, 5);
	// 	assert_eq!(rb.nodes[0].success_count, 7);
	// 	assert_eq!(rb.nodes[1].failure_count, 666);
	// 	assert_eq!(rb.nodes[1].success_count, 667);
	// 	assert_eq!(rb.nodes[2].failure_count, 0);
	// 	assert_eq!(rb.nodes[2].success_count, 0);
	// 	assert_eq!(rb.nodes[3].failure_count, 0);
	// 	assert_eq!(rb.nodes[3].success_count, 0);
	// 	assert_eq!(rb.nodes[4].failure_count, 0);
	// 	assert_eq!(rb.nodes[4].success_count, 0);
	// 	assert_eq!(rb.nodes[5].failure_count, 0);
	// 	assert_eq!(rb.nodes[5].success_count, 0);
	// 	assert_eq!(rb.nodes[6].failure_count, 0);
	// 	assert_eq!(rb.nodes[6].success_count, 0);
	// 	assert_eq!(rb.nodes[7].failure_count, 666);
	// 	assert_eq!(rb.nodes[7].success_count, 42);

	// 	// we record something into node 6 and fill up the buffer
	// 	rb.last_record = start_time + Duration::from_millis(6000);
	// 	rb.nodes[6].failure_count = 5;
	// 	rb.nodes[6].success_count = 5;
	// 	rb.nodes[7].failure_count = 5;
	// 	rb.nodes[7].success_count = 5;
	// 	rb.nodes[8].failure_count = 5;
	// 	rb.nodes[8].success_count = 5;
	// 	rb.nodes[9].failure_count = 5;
	// 	rb.nodes[9].success_count = 5;

	// 	// now we skip to node 8 and check the skipped nodes that needed to be reset
	// 	// Node 6 => Node 8
	// 	assert_eq!(rb.get_cursor(buffer_span_duration, start_time + Duration::from_secs(8)), 8);
	// 	assert_eq!(rb.nodes[0].failure_count, 5);
	// 	assert_eq!(rb.nodes[0].success_count, 7);
	// 	assert_eq!(rb.nodes[6].failure_count, 5);
	// 	assert_eq!(rb.nodes[6].success_count, 5);
	// 	assert_eq!(rb.nodes[7].failure_count, 0);
	// 	assert_eq!(rb.nodes[7].success_count, 0);
	// 	assert_eq!(rb.nodes[8].failure_count, 0);
	// 	assert_eq!(rb.nodes[8].success_count, 0);
	// 	assert_eq!(rb.nodes[9].failure_count, 5);
	// 	assert_eq!(rb.nodes[9].success_count, 5);
	// 	// Now we do the very same thing again to make sure get_cursor always returns the same thing even when called multiple times
	// 	assert_eq!(rb.get_cursor(buffer_span_duration, start_time + Duration::from_secs(8)), 8);
	// 	assert_eq!(rb.nodes[0].failure_count, 5);
	// 	assert_eq!(rb.nodes[0].success_count, 7);
	// 	assert_eq!(rb.nodes[6].failure_count, 5);
	// 	assert_eq!(rb.nodes[6].success_count, 5);
	// 	assert_eq!(rb.nodes[7].failure_count, 0);
	// 	assert_eq!(rb.nodes[7].success_count, 0);
	// 	assert_eq!(rb.nodes[8].failure_count, 0);
	// 	assert_eq!(rb.nodes[8].success_count, 0);
	// 	assert_eq!(rb.nodes[9].failure_count, 5);
	// 	assert_eq!(rb.nodes[9].success_count, 5);

	// 	rb.nodes[0].failure_count = 5;
	// 	rb.nodes[0].success_count = 5;
	// 	rb.nodes[1].failure_count = 5;
	// 	rb.nodes[1].success_count = 5;
	// 	rb.nodes[2].failure_count = 5;
	// 	rb.nodes[2].success_count = 5;
	// 	rb.nodes[3].failure_count = 5;
	// 	rb.nodes[3].success_count = 5;
	// 	rb.nodes[4].failure_count = 5;
	// 	rb.nodes[4].success_count = 5;
	// 	rb.nodes[5].failure_count = 5;
	// 	rb.nodes[5].success_count = 5;

	// 	// now we skip five buffer durations ahead and test that the ring buffer went in a circle
	// 	// Node 9 => Node 4
	// 	rb.last_record = start_time + Duration::from_millis(8000);
	// 	assert_eq!(rb.get_cursor(buffer_span_duration, start_time + Duration::from_secs(14)), 4);
	// 	assert_eq!(rb.nodes[9].failure_count, 0);
	// 	assert_eq!(rb.nodes[9].success_count, 0);
	// 	assert_eq!(rb.nodes[0].failure_count, 0);
	// 	assert_eq!(rb.nodes[0].success_count, 0);
	// 	assert_eq!(rb.nodes[1].failure_count, 0);
	// 	assert_eq!(rb.nodes[1].success_count, 0);
	// 	assert_eq!(rb.nodes[2].failure_count, 0);
	// 	assert_eq!(rb.nodes[2].success_count, 0);
	// 	assert_eq!(rb.nodes[3].failure_count, 0);
	// 	assert_eq!(rb.nodes[3].success_count, 0);
	// 	assert_eq!(rb.nodes[4].failure_count, 0);
	// 	assert_eq!(rb.nodes[4].success_count, 0);
	// 	assert_eq!(rb.nodes[5].failure_count, 5);
	// 	assert_eq!(rb.nodes[5].success_count, 5);

	// 	rb.nodes[0].failure_count = 5;
	// 	rb.nodes[0].success_count = 5;
	// 	rb.nodes[1].failure_count = 5;
	// 	rb.nodes[1].success_count = 5;
	// 	rb.nodes[2].failure_count = 5;
	// 	rb.nodes[2].success_count = 5;
	// 	rb.nodes[3].failure_count = 5;
	// 	rb.nodes[3].success_count = 5;
	// 	rb.nodes[4].failure_count = 5;
	// 	rb.nodes[4].success_count = 5;
	// 	rb.nodes[5].failure_count = 5;
	// 	rb.nodes[5].success_count = 5;
	// 	rb.nodes[6].failure_count = 5;
	// 	rb.nodes[6].success_count = 5;
	// 	rb.nodes[7].failure_count = 5;
	// 	rb.nodes[7].success_count = 5;
	// 	rb.nodes[8].failure_count = 5;
	// 	rb.nodes[8].success_count = 5;
	// 	rb.nodes[9].failure_count = 5;
	// 	rb.nodes[9].success_count = 5;

	// 	// and lastely we skip a bunch of time ahead that is more than a couple round trips around the ring buffer
	// 	// and make sure everything was reset
	// 	assert_eq!(rb.get_cursor(buffer_span_duration, start_time + Duration::from_secs(605)), 5);
	// 	assert_eq!(rb.nodes[0].failure_count, 0);
	// 	assert_eq!(rb.nodes[0].success_count, 0);
	// 	assert_eq!(rb.nodes[1].failure_count, 0);
	// 	assert_eq!(rb.nodes[1].success_count, 0);
	// 	assert_eq!(rb.nodes[2].failure_count, 0);
	// 	assert_eq!(rb.nodes[2].success_count, 0);
	// 	assert_eq!(rb.nodes[3].failure_count, 0);
	// 	assert_eq!(rb.nodes[3].success_count, 0);
	// 	assert_eq!(rb.nodes[4].failure_count, 0);
	// 	assert_eq!(rb.nodes[4].success_count, 0);
	// 	assert_eq!(rb.nodes[5].failure_count, 0);
	// 	assert_eq!(rb.nodes[5].success_count, 0);
	// 	assert_eq!(rb.nodes[6].failure_count, 0);
	// 	assert_eq!(rb.nodes[6].success_count, 0);
	// 	assert_eq!(rb.nodes[7].failure_count, 0);
	// 	assert_eq!(rb.nodes[7].success_count, 0);
	// 	assert_eq!(rb.nodes[8].failure_count, 0);
	// 	assert_eq!(rb.nodes[8].success_count, 0);
	// 	assert_eq!(rb.nodes[9].failure_count, 0);
	// 	assert_eq!(rb.nodes[9].success_count, 0);
	// }

	// #[test]
	// fn add_failure_success_test() {
	// 	let mut buffer = RingBuffer::new(1);

	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 0);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);
	// 	buffer.add_failure(Duration::from_secs(10), Instant::now());
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 1);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);
	// 	buffer.add_success(Duration::from_secs(10), Instant::now());
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 1);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 1);
	// 	buffer.add_success(Duration::from_secs(10), Instant::now());
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 1);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 2);
	// 	buffer.add_failure(Duration::from_secs(10), Instant::now());
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 2);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 2);
	// }

	// #[test]
	// fn next_add_failure_success_test() {
	// 	let mut buffer = RingBuffer::new(5);
	// 	let buffer_span_duration = Duration::from_secs(10);

	// 	// we start with the cursor pointing to node 0 and make sure we count each success and failure
	// 	let _ = buffer.get_cursor(buffer_span_duration, Instant::now());
	// 	assert_eq!(buffer.cursor, 0);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 0);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);
	// 	buffer.add_failure(buffer_span_duration, Instant::now());
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 1);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);
	// 	buffer.add_failure(buffer_span_duration, Instant::now());
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 2);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);

	// 	// now we skip forward in time to check the next node
	// 	buffer.start_time = Instant::now() - (buffer_span_duration * 1) - Duration::from_secs(1);
	// 	// we check the cursor to make sure we evaluate the state
	// 	// calling `add_success` or `add_failure` will check the cursor automatically
	// 	let _ = buffer.get_cursor(buffer_span_duration, Instant::now());

	// 	assert_eq!(buffer.get_node_info(0).failure_count, 2); // we retained the data of old nodes
	// 	assert_eq!(buffer.get_node_info(0).success_count, 0);
	// 	assert_eq!(buffer.cursor, 1);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 0);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);
	// 	buffer.add_success(buffer_span_duration, Instant::now());
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 0);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 1);
	// 	buffer.add_success(buffer_span_duration, Instant::now());
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 0);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 2);

	// 	// now we skip ahead again to make sure we get to a new node
	// 	buffer.start_time = Instant::now() - (buffer_span_duration * 2) - Duration::from_secs(1);
	// 	let _ = buffer.get_cursor(buffer_span_duration, Instant::now());

	// 	assert_eq!(buffer.get_node_info(0).failure_count, 2);
	// 	assert_eq!(buffer.get_node_info(0).success_count, 0);
	// 	assert_eq!(buffer.get_node_info(1).failure_count, 0);
	// 	assert_eq!(buffer.get_node_info(1).success_count, 2);
	// 	assert_eq!(buffer.cursor, 2);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 0);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);
	// 	buffer.add_success(buffer_span_duration, Instant::now());
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 0);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 1);
	// 	buffer.add_failure(buffer_span_duration, Instant::now());
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 1);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 1);

	// 	// this time we skip one node and populate the skipped node with data to make sure we clear skipped nodes
	// 	buffer.start_time = Instant::now() - (buffer_span_duration * 4) - Duration::from_secs(1);
	// 	buffer.last_record = Instant::now() - (buffer_span_duration * 1) - Duration::from_secs(1);
	// 	buffer.nodes[3].failure_count = 42;
	// 	buffer.nodes[3].success_count = 666;
	// 	let _ = buffer.get_cursor(buffer_span_duration, Instant::now());

	// 	assert_eq!(buffer.get_node_info(0).failure_count, 2);
	// 	assert_eq!(buffer.get_node_info(0).success_count, 0);
	// 	assert_eq!(buffer.get_node_info(1).failure_count, 0);
	// 	assert_eq!(buffer.get_node_info(1).success_count, 2);
	// 	assert_eq!(buffer.get_node_info(2).failure_count, 1);
	// 	assert_eq!(buffer.get_node_info(2).success_count, 1);
	// 	assert_eq!(buffer.get_node_info(3).failure_count, 0); // reset because it was skipped
	// 	assert_eq!(buffer.get_node_info(3).success_count, 0); // reset because it was skipped
	// 	assert_eq!(buffer.cursor, 4);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 0);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);
	// 	buffer.add_failure(buffer_span_duration, Instant::now());
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).failure_count, 1);
	// 	assert_eq!(buffer.get_node_info(buffer.cursor).success_count, 0);
	// }

	// #[test]
	// fn get_elapsed_time_test() {
	// 	let timeout = Instant::now();
	// 	let buffer = RingBuffer {
	// 		cursor: 0,
	// 		nodes: vec![Node {
	// 			failure_count: 0,
	// 			success_count: 0,
	// 		}],
	// 		start_time: timeout,
	// 		last_record: timeout,
	// 	};

	// 	assert_eq!(
	// 		buffer.get_elapsed_time(Duration::from_secs(5), timeout + Duration::from_secs(1)),
	// 		Duration::from_secs(1)
	// 	);
	// 	assert_eq!(
	// 		buffer.get_elapsed_time(Duration::from_secs(5), timeout + Duration::from_secs(4)),
	// 		Duration::from_secs(4)
	// 	);
	// 	assert_eq!(
	// 		buffer.get_elapsed_time(Duration::from_secs(5), timeout + Duration::from_secs(5)),
	// 		Duration::from_secs(0)
	// 	);
	// 	assert_eq!(
	// 		buffer.get_elapsed_time(Duration::from_secs(5), timeout + Duration::from_secs(6)),
	// 		Duration::from_secs(1)
	// 	);
	// }

	// #[test]
	// fn get_error_rate_test() {
	// 	let buffer = RingBuffer {
	// 		cursor: 0,
	// 		nodes: vec![
	// 			Node {
	// 				failure_count: 50,
	// 				success_count: 50,
	// 			},
	// 			Node {
	// 				failure_count: 0,
	// 				success_count: 0,
	// 			},
	// 		],
	// 		start_time: Instant::now(),
	// 		last_record: Instant::now(),
	// 	};
	// 	assert_eq!(buffer.get_error_rate(10), 0.0); // cursor on first node

	// 	let buffer = RingBuffer {
	// 		cursor: 1,
	// 		nodes: vec![
	// 			Node {
	// 				failure_count: 50,
	// 				success_count: 50,
	// 			},
	// 			Node {
	// 				failure_count: 0,
	// 				success_count: 0,
	// 			},
	// 		],
	// 		start_time: Instant::now(),
	// 		last_record: Instant::now(),
	// 	};
	// 	assert_eq!(buffer.get_error_rate(10), 50.0); // 50 of 100 = 50%

	// 	let buffer = RingBuffer {
	// 		cursor: 0,
	// 		nodes: vec![
	// 			Node {
	// 				failure_count: 0,
	// 				success_count: 0,
	// 			},
	// 			Node {
	// 				failure_count: 50,
	// 				success_count: 50,
	// 			},
	// 			Node {
	// 				failure_count: 10,
	// 				success_count: 90,
	// 			},
	// 		],
	// 		start_time: Instant::now(),
	// 		last_record: Instant::now(),
	// 	};
	// 	assert_eq!(buffer.get_error_rate(10), 30.0); // 60 of 200 = 30%

	// 	let buffer = RingBuffer {
	// 		cursor: 0,
	// 		nodes: vec![
	// 			Node {
	// 				failure_count: 0,
	// 				success_count: 0,
	// 			},
	// 			Node {
	// 				failure_count: 5,
	// 				success_count: 5,
	// 			},
	// 			Node {
	// 				failure_count: 1,
	// 				success_count: 9,
	// 			},
	// 		],
	// 		start_time: Instant::now(),
	// 		last_record: Instant::now(),
	// 	};
	// 	assert_eq!(buffer.get_error_rate(100), 0.0); // 6 of 20 = 30% but less than min_eval_size
	// }

	// #[test]
	// fn get_node_info_test() {
	// 	let buffer = RingBuffer {
	// 		cursor: 0,
	// 		nodes: vec![
	// 			Node {
	// 				failure_count: 42,
	// 				success_count: 666,
	// 			},
	// 			Node {
	// 				failure_count: 0,
	// 				success_count: 42,
	// 			},
	// 			Node {
	// 				failure_count: 256,
	// 				success_count: 0,
	// 			},
	// 		],
	// 		start_time: Instant::now(),
	// 		last_record: Instant::now(),
	// 	};

	// 	assert_eq!(
	// 		buffer.get_node_info(0),
	// 		NodeInfo {
	// 			failure_count: 42,
	// 			success_count: 666,
	// 		}
	// 	);
	// 	assert_eq!(
	// 		buffer.get_node_info(1),
	// 		NodeInfo {
	// 			failure_count: 0,
	// 			success_count: 42,
	// 		}
	// 	);
	// 	assert_eq!(
	// 		buffer.get_node_info(2),
	// 		NodeInfo {
	// 			failure_count: 256,
	// 			success_count: 0,
	// 		}
	// 	);
	// }
}
