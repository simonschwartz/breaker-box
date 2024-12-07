use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct Node {
	expires: Instant,
	error_count: usize,
	total_count: usize,
}

impl Node {
	pub fn new() -> Self {
		Self {
			expires: Instant::now(),
			error_count: 0,
			total_count: 0,
		}
	}

	pub fn reset(&mut self) {
		self.expires = Instant::now();
		self.error_count = 0;
		self.total_count = 0;
	}
}

#[derive(Debug)]
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

	pub fn get_cursor(&self) -> &Node {
		&self.nodes[self.cursor]
	}

	pub fn add_error(&mut self) {
		self.nodes[self.cursor].error_count += 1;
		self.nodes[self.cursor].total_count += 1;
	}

	pub fn add_success(&mut self) {
		self.nodes[self.cursor].total_count += 1;
	}

	pub fn next(&mut self) {
		if self.cursor == self.nodes.len() - 1 {
			self.cursor = 0;
		} else {
			self.cursor += 1;
		}
		self.nodes[self.cursor].reset();
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn new_test() {
		assert_eq!(RingBuffer::new(1).nodes.len(), 1);
		assert_eq!(RingBuffer::new(1).nodes[0].error_count, 0);
		assert_eq!(RingBuffer::new(1).nodes[0].total_count, 0);
		assert_eq!(RingBuffer::new(5).nodes.len(), 5);
		assert_eq!(RingBuffer::new(5).nodes[4].error_count, 0);
		assert_eq!(RingBuffer::new(5).nodes[4].total_count, 0);
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
		assert_eq!(RingBuffer::new(1).get_cursor().error_count, 0);
		assert_eq!(RingBuffer::new(1).get_cursor().total_count, 0);
	}

	#[test]
	fn add_result_test() {
		let mut buffer = RingBuffer::new(1);

		assert_eq!(buffer.get_cursor().error_count, 0);
		assert_eq!(buffer.get_cursor().total_count, 0);
		buffer.add_error();
		assert_eq!(buffer.get_cursor().error_count, 1);
		assert_eq!(buffer.get_cursor().total_count, 1);
		buffer.add_success();
		assert_eq!(buffer.get_cursor().error_count, 1);
		assert_eq!(buffer.get_cursor().total_count, 2);
		buffer.add_success();
		assert_eq!(buffer.get_cursor().error_count, 1);
		assert_eq!(buffer.get_cursor().total_count, 3);
		buffer.add_error();
		assert_eq!(buffer.get_cursor().error_count, 2);
		assert_eq!(buffer.get_cursor().total_count, 4);
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
	fn next_add_result_test() {
		let mut buffer = RingBuffer::new(3);
		assert_eq!(buffer.cursor, 0);
		assert_eq!(buffer.get_cursor().error_count, 0);
		assert_eq!(buffer.get_cursor().total_count, 0);
		buffer.add_error();
		assert_eq!(buffer.get_cursor().error_count, 1);
		assert_eq!(buffer.get_cursor().total_count, 1);
		buffer.add_error();
		assert_eq!(buffer.get_cursor().error_count, 2);
		assert_eq!(buffer.get_cursor().total_count, 2);
		buffer.next();
		assert_eq!(buffer.get_cursor().error_count, 0);
		assert_eq!(buffer.get_cursor().total_count, 0);
		buffer.add_success();
		assert_eq!(buffer.get_cursor().error_count, 0);
		assert_eq!(buffer.get_cursor().total_count, 1);
		buffer.add_success();
		assert_eq!(buffer.get_cursor().error_count, 0);
		assert_eq!(buffer.get_cursor().total_count, 2);
		buffer.next();
		assert_eq!(buffer.get_cursor().error_count, 0);
		assert_eq!(buffer.get_cursor().total_count, 0);
		buffer.add_success();
		assert_eq!(buffer.get_cursor().error_count, 0);
		assert_eq!(buffer.get_cursor().total_count, 1);
		buffer.add_error();
		assert_eq!(buffer.get_cursor().error_count, 1);
		assert_eq!(buffer.get_cursor().total_count, 2);
		buffer.next();
		assert_eq!(buffer.get_cursor().error_count, 0);
		assert_eq!(buffer.get_cursor().total_count, 0);
	}
}