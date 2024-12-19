use crate::ring_buffer::{NodeInfo, RingBuffer};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Buffer {
	Index(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MiddleBuffer {
	One(Buffer),
	Two(Buffer, Buffer),
}

#[derive(Debug, PartialEq)]
pub struct Visualizer<'a> {
	buffer: &'a RingBuffer,
	top: Vec<Buffer>,
	middle: Option<Vec<MiddleBuffer>>,
	bottom: Option<Vec<Buffer>>,
}

impl<'a> Visualizer<'a> {
	pub fn new(buffer: &'a RingBuffer) -> Self {
		match buffer.get_length() {
			0 => panic!("Must have at least one buffer enabled"),
			1 => Self {
				buffer,
				top: vec![Buffer::Index(0)],
				middle: None,
				bottom: None,
			},
			2 => Self {
				buffer,
				top: vec![Buffer::Index(0), Buffer::Index(1)],
				middle: None,
				bottom: None,
			},
			3 => Self {
				buffer,
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: None,
			},
			4 => Self {
				buffer,
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: Some(vec![Buffer::Index(3)]),
			},
			5 => Self {
				buffer,
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: Some(vec![Buffer::Index(3), Buffer::Index(4)]),
			},
			6 => Self {
				buffer,
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: Some(vec![Buffer::Index(3), Buffer::Index(4), Buffer::Index(5)]),
			},
			length => {
				let n = (length - 7) / 2;
				let mut middle = Vec::new();
				for index in (0..n).step_by(2) {
					middle.push(MiddleBuffer::Two(Buffer::Index(index), Buffer::Index(index + 1)));
				}
				let rest = length - n - 6;
				middle.push(if length % 2 == 0 {
					MiddleBuffer::Two(Buffer::Index(rest), Buffer::Index(rest + 1))
				} else {
					MiddleBuffer::One(Buffer::Index(rest))
				});

				Self {
					buffer,
					top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
					middle: Some(middle),
					bottom: Some(vec![
						Buffer::Index(length - 2),
						Buffer::Index(length - 1),
						Buffer::Index(length),
					]),
				}
			},
		}
	}

	fn render_top(&self, index: usize) -> String {
		let is_active = self.buffer.get_cursor() == index;
		match is_active {
			true => String::from("┏━━━━━━━━━━━━━━━━━┓"),
			false => String::from("┌─────────────────┐"),
		}
	}

	fn render_middle(&self, index: usize) -> String {
		let is_active = self.buffer.get_cursor() == index;
		let infos = self.buffer.get_node_info(index);
		match is_active {
			true => format!(
				"┃ B{index:<2} \x1b[42m {:0>3} \x1b[0m \x1b[41m {:0>3} \x1b[0m ┃",
				infos.total_count, infos.failure_count
			),
			false => format!(
				"│ B{index:<2} \x1b[42m {:0>3} \x1b[0m \x1b[41m {:0>3} \x1b[0m │",
				infos.total_count, infos.failure_count
			),
		}
	}

	fn render_bottom(&self, index: usize) -> String {
		let is_active = self.buffer.get_cursor() == index;
		match is_active {
			true => String::from("┗━━━━━━━━━━━━━━━━━┛"),
			false => String::from("└─────────────────┘"),
		}
	}

	pub fn render(&self) -> String {
		let mut top = [String::new(), String::new(), String::new()];
		let mut middle = [String::new(), String::new(), String::new()];
		let mut bottom = [String::new(), String::new(), String::new()];

		for index in 0..self.top.len() {
			top[0].push_str(&self.render_top(index));
			top[1].push_str(&self.render_middle(index));
			top[2].push_str(&self.render_bottom(index));
			if index < self.top.len() - 1 {
				top[0].push_str("  ");
				top[1].push_str("─▶");
				top[2].push_str("  ");
			}
		}

		if self.top.len() < 3 {
			let repetition = 3 - self.top.len();
			top[0].push_str(&"                   ".repeat(repetition));

			match repetition {
				1 => {
					top[1].push_str("──────────────────┐");
					top[2].push_str("                  │");
				},
				2 => {
					top[1].push_str("─────────────────────────────────────┐");
					top[2].push_str("                                     │");
				},
				_ => unreachable!(
					"The number has to be between 1 and 2 due to the if condition and the panic at 0 in the new method"
				),
			}
		}

		let mut output = top.join("\n");
		output.push_str(&middle.join("\n"));
		output.push_str(&bottom.join("\n"));
		output
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn new_test() {
		assert_eq!(
			Visualizer::new(&RingBuffer::new(1)),
			Visualizer {
				buffer: &RingBuffer::new(1),
				top: vec![Buffer::Index(0)],
				middle: None,
				bottom: None,
			}
		);
		assert_eq!(
			Visualizer::new(&RingBuffer::new(2)),
			Visualizer {
				buffer: &RingBuffer::new(2),
				top: vec![Buffer::Index(0), Buffer::Index(1)],
				middle: None,
				bottom: None,
			}
		);
		assert_eq!(
			Visualizer::new(&RingBuffer::new(3)),
			Visualizer {
				buffer: &RingBuffer::new(3),
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: None,
			}
		);
		assert_eq!(
			Visualizer::new(&RingBuffer::new(4)),
			Visualizer {
				buffer: &RingBuffer::new(4),
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: Some(vec![Buffer::Index(3)]),
			}
		);
		assert_eq!(
			Visualizer::new(&RingBuffer::new(5)),
			Visualizer {
				buffer: &RingBuffer::new(5),
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: Some(vec![Buffer::Index(3), Buffer::Index(4)]),
			}
		);
		assert_eq!(
			Visualizer::new(&RingBuffer::new(6)),
			Visualizer {
				buffer: &RingBuffer::new(6),
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: Some(vec![Buffer::Index(3), Buffer::Index(4), Buffer::Index(5)]),
			}
		);
		assert_eq!(
			Visualizer::new(&RingBuffer::new(7)),
			Visualizer {
				buffer: &RingBuffer::new(7),
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: Some(vec![MiddleBuffer::One(Buffer::Index(3))]),
				bottom: Some(vec![Buffer::Index(4), Buffer::Index(5), Buffer::Index(6)]),
			}
		);
		assert_eq!(
			Visualizer::new(&RingBuffer::new(8)),
			Visualizer {
				buffer: &RingBuffer::new(8),
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: Some(vec![MiddleBuffer::Two(Buffer::Index(3), Buffer::Index(4))]),
				bottom: Some(vec![Buffer::Index(5), Buffer::Index(6), Buffer::Index(7)]),
			}
		);
		assert_eq!(
			Visualizer::new(&RingBuffer::new(9)),
			Visualizer {
				buffer: &RingBuffer::new(9),
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: Some(vec![
					MiddleBuffer::Two(Buffer::Index(3), Buffer::Index(4)),
					MiddleBuffer::One(Buffer::Index(5)),
				]),
				bottom: Some(vec![Buffer::Index(6), Buffer::Index(7), Buffer::Index(8)]),
			}
		);
		assert_eq!(
			Visualizer::new(&RingBuffer::new(10)),
			Visualizer {
				buffer: &RingBuffer::new(10),
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: Some(vec![
					MiddleBuffer::Two(Buffer::Index(3), Buffer::Index(4)),
					MiddleBuffer::Two(Buffer::Index(5), Buffer::Index(6)),
					MiddleBuffer::One(Buffer::Index(7)),
				]),
				bottom: Some(vec![Buffer::Index(8), Buffer::Index(9), Buffer::Index(10)]),
			}
		);
		assert_eq!(
			Visualizer::new(&RingBuffer::new(11)),
			Visualizer {
				buffer: &RingBuffer::new(11),
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: Some(vec![
					MiddleBuffer::Two(Buffer::Index(3), Buffer::Index(4)),
					MiddleBuffer::Two(Buffer::Index(5), Buffer::Index(6)),
					MiddleBuffer::Two(Buffer::Index(7), Buffer::Index(8)),
				]),
				bottom: Some(vec![Buffer::Index(9), Buffer::Index(10), Buffer::Index(11)]),
			}
		);
		assert_eq!(
			Visualizer::new(&RingBuffer::new(12)),
			Visualizer {
				buffer: &RingBuffer::new(12),
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: Some(vec![
					MiddleBuffer::Two(Buffer::Index(3), Buffer::Index(4)),
					MiddleBuffer::Two(Buffer::Index(5), Buffer::Index(6)),
					MiddleBuffer::Two(Buffer::Index(7), Buffer::Index(8)),
					MiddleBuffer::One(Buffer::Index(9)),
				]),
				bottom: Some(vec![Buffer::Index(10), Buffer::Index(11), Buffer::Index(12)]),
			}
		);
		assert_eq!(
			Visualizer::new(&RingBuffer::new(13)),
			Visualizer {
				buffer: &RingBuffer::new(13),
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: Some(vec![
					MiddleBuffer::Two(Buffer::Index(3), Buffer::Index(4)),
					MiddleBuffer::Two(Buffer::Index(5), Buffer::Index(6)),
					MiddleBuffer::Two(Buffer::Index(7), Buffer::Index(8)),
					MiddleBuffer::Two(Buffer::Index(9), Buffer::Index(10)),
				]),
				bottom: Some(vec![Buffer::Index(11), Buffer::Index(12), Buffer::Index(13)]),
			}
		);
	}
}
