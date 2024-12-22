use crate::ring_buffer::RingBuffer;

#[derive(Debug, Clone, Copy, PartialEq)]
enum MiddleBuffer {
	One(usize),
	Two(usize, usize),
}

#[derive(Debug, PartialEq)]
pub struct Visualizer<'a> {
	buffer: &'a RingBuffer,
	top: Vec<usize>,
	middle: Option<Vec<MiddleBuffer>>,
	bottom: Option<Vec<usize>>,
}

impl<'a> Visualizer<'a> {
	pub fn new(buffer: &'a RingBuffer) -> Self {
		match buffer.get_length() {
			0 => panic!("Must have at least one buffer enabled"),
			1 => Self {
				buffer,
				top: vec![0],
				middle: None,
				bottom: None,
			},
			2 => Self {
				buffer,
				top: vec![0, 1],
				middle: None,
				bottom: None,
			},
			3 => Self {
				buffer,
				top: vec![0, 1, 2],
				middle: None,
				bottom: None,
			},
			4 => Self {
				buffer,
				top: vec![0, 1, 2],
				middle: None,
				bottom: Some(vec![3]),
			},
			5 => Self {
				buffer,
				top: vec![0, 1, 2],
				middle: None,
				bottom: Some(vec![4, 3]),
			},
			6 => Self {
				buffer,
				top: vec![0, 1, 2],
				middle: None,
				bottom: Some(vec![5, 4, 3]),
			},
			length => {
				let offset = (length - 7) / 2; // safe because we are in a match with length > 6
				let largest = 6 + offset;
				let bottom = vec![largest, largest - 1, largest - 2];

				let mut asc = Vec::with_capacity(length - 3); // safe because we are in a match with length > 6
				for i in 3..length {
					if !bottom.contains(&i) {
						asc.push(i);
					}
				}

				let mut middle_buffers = Vec::with_capacity(asc.len() / 2 + 1);
				let mut small = 0;
				let mut large = asc.len() - 1; // safe because we are in a match with length > 6

				while small <= large {
					let large_val = asc[large];
					if small == large {
						middle_buffers.push(MiddleBuffer::One(large_val));
						break;
					} else {
						let small_val = asc[small];
						middle_buffers.push(MiddleBuffer::Two(large_val, small_val));
						small += 1;
						large -= 1;
					}
				}

				Self {
					buffer,
					top: vec![0, 1, 2],
					middle: Some(middle_buffers),
					bottom: Some(bottom),
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
		let mut middle = vec![String::new(), String::new()];
		let mut bottom = [String::new(), String::new(), String::new()];

		// TOP
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
			match repetition {
				1 => {
					top[1].push_str("───────────┐");
					top[2].push_str("           │");
				},
				2 => {
					top[1].push_str("────────────────────────────────┐");
					top[2].push_str("                                │");
				},
				_ => unreachable!(
					"The number has to be between 1 and 2 due to the if condition and the panic at 0 in the new method"
				),
			}
		}

		// MIDDLE
		match &self.middle {
			None => {
				if self.bottom.is_some() {
					middle[0].push_str("         ▲                                         │");
					middle[1].push_str("         │                                         ▼");
				} else {
					middle[0].push_str("         ▲                                         │");
					middle[1].push_str("         └─────────────────────────────────────────┘");
				}
			},
			Some(nodes) => {
				middle[0].push_str("         ▲                                         │");
				middle[1].push_str("         │                                         ▼");
				let mut i = 1;
				for node in nodes {
					middle.extend([
						String::new(),
						String::new(),
						String::new(),
						String::new(),
						String::new(),
					]);
					match node {
						MiddleBuffer::One(index1) => {
							middle[i + 1]
								.push_str(&format!("         │                                {}", self.render_top(*index1)));
							middle[i + 2]
								.push_str(&format!("         │                                {}", self.render_middle(*index1)));
							middle[i + 3]
								.push_str(&format!("         │                                {}", self.render_bottom(*index1)));
							middle[i + 4].push_str("         │                                         │");
							middle[i + 5].push_str("         │                                         ▼");
							i += 5;
						},
						MiddleBuffer::Two(index1, index2) => {
							middle[i + 1].push_str(&format!(
								"{}                       {}",
								self.render_top(*index1),
								self.render_top(*index2)
							));
							middle[i + 2].push_str(&format!(
								"{}                       {}",
								self.render_middle(*index1),
								self.render_middle(*index2)
							));
							middle[i + 3].push_str(&format!(
								"{}                       {}",
								self.render_bottom(*index1),
								self.render_bottom(*index2)
							));
							middle[i + 4].push_str("         ▲                                         │");
							middle[i + 5].push_str("         │                                         ▼");
							i += 5;
						},
					}
				}
			},
		}

		// BOTTOM
		match &self.bottom {
			None => {},
			Some(b) => {
				if b.len() < 3 {
					let repetition = 3 - b.len();
					bottom[2].push_str(&"                     ".repeat(repetition));

					match repetition {
						0 => {},
						1 => {
							bottom[0].push_str("         │           ");
							bottom[1].push_str("         └───────────");
						},
						2 => {
							bottom[0].push_str("         │                                ");
							bottom[1].push_str("         └────────────────────────────────");
						},
						_ => unreachable!("The number has to be between 0 and 2 due to the if condition"),
					}
				}

				for index in b {
					bottom[0].push_str(&self.render_top(*index));
					bottom[1].push_str(&self.render_middle(*index));
					bottom[2].push_str(&self.render_bottom(*index));
					if *index != b[b.len() - 1] {
						bottom[0].push_str("  ");
						bottom[1].push_str("◀─");
						bottom[2].push_str("  ");
					}
				}
			},
		}

		let mut output = top.join("\n");
		output.push('\n');
		output.push_str(&middle.join("\n"));
		output.push('\n');
		output.push_str(&bottom.join("\n"));
		output
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn new_test() {
		let rb = RingBuffer::new(1);
		assert_eq!(Visualizer::new(&rb).top, vec![0]);
		assert_eq!(Visualizer::new(&rb).middle, None);
		assert_eq!(Visualizer::new(&rb).bottom, None);

		let rb = RingBuffer::new(2);
		assert_eq!(Visualizer::new(&rb).top, vec![0, 1]);
		assert_eq!(Visualizer::new(&rb).middle, None);
		assert_eq!(Visualizer::new(&rb).bottom, None);

		let rb = RingBuffer::new(3);
		assert_eq!(Visualizer::new(&rb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&rb).middle, None);
		assert_eq!(Visualizer::new(&rb).bottom, None);

		let rb = RingBuffer::new(4);
		assert_eq!(Visualizer::new(&rb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&rb).middle, None);
		assert_eq!(Visualizer::new(&rb).bottom, Some(vec![3]));

		let rb = RingBuffer::new(5);
		assert_eq!(Visualizer::new(&rb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&rb).middle, None);
		assert_eq!(Visualizer::new(&rb).bottom, Some(vec![4, 3]));

		let rb = RingBuffer::new(6);
		assert_eq!(Visualizer::new(&rb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&rb).middle, None);
		assert_eq!(Visualizer::new(&rb).bottom, Some(vec![5, 4, 3]));

		let rb = RingBuffer::new(7);
		assert_eq!(Visualizer::new(&rb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&rb).middle, Some(vec![MiddleBuffer::One(3)]));
		assert_eq!(Visualizer::new(&rb).bottom, Some(vec![6, 5, 4]));

		let rb = RingBuffer::new(8);
		assert_eq!(Visualizer::new(&rb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&rb).middle, Some(vec![MiddleBuffer::Two(7, 3)]));
		assert_eq!(Visualizer::new(&rb).bottom, Some(vec![6, 5, 4]));

		let rb = RingBuffer::new(9);
		assert_eq!(Visualizer::new(&rb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&rb).middle, Some(vec![MiddleBuffer::Two(8, 3), MiddleBuffer::One(4),]));
		assert_eq!(Visualizer::new(&rb).bottom, Some(vec![7, 6, 5]));

		let rb = RingBuffer::new(10);
		assert_eq!(Visualizer::new(&rb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&rb).middle, Some(vec![MiddleBuffer::Two(9, 3), MiddleBuffer::Two(8, 4),]));
		assert_eq!(Visualizer::new(&rb).bottom, Some(vec![7, 6, 5]));

		let rb = RingBuffer::new(11);
		assert_eq!(Visualizer::new(&rb).top, vec![0, 1, 2]);
		assert_eq!(
			Visualizer::new(&rb).middle,
			Some(vec![MiddleBuffer::Two(10, 3), MiddleBuffer::Two(9, 4), MiddleBuffer::One(5),])
		);
		assert_eq!(Visualizer::new(&rb).bottom, Some(vec![8, 7, 6]));

		let rb = RingBuffer::new(12);
		assert_eq!(Visualizer::new(&rb).top, vec![0, 1, 2]);
		assert_eq!(
			Visualizer::new(&rb).middle,
			Some(vec![
				MiddleBuffer::Two(11, 3),
				MiddleBuffer::Two(10, 4),
				MiddleBuffer::Two(9, 5),
			])
		);
		assert_eq!(Visualizer::new(&rb).bottom, Some(vec![8, 7, 6]));

		let rb = RingBuffer::new(13);
		assert_eq!(Visualizer::new(&rb).top, vec![0, 1, 2]);
		assert_eq!(
			Visualizer::new(&rb).middle,
			Some(vec![
				MiddleBuffer::Two(12, 3),
				MiddleBuffer::Two(11, 4),
				MiddleBuffer::Two(10, 5),
				MiddleBuffer::One(6),
			])
		);
		assert_eq!(Visualizer::new(&rb).bottom, Some(vec![9, 8, 7]));
	}
}
