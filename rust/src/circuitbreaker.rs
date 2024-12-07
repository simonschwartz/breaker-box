use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct Node {
	pub expires: Instant,
	pub error_count: usize,
	pub total_count: usize,
}

impl Node {
	pub fn new() -> Self {
		Self {
			expires: Instant::now(),
			error_count: 0,
			total_count: 0,
		}
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

	pub fn next(&mut self) {
		if self.cursor == self.nodes.len() - 1 {
			self.cursor = 0;
		} else {
			self.cursor += 1;
		}
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
	fn netxt_test() {
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
}
