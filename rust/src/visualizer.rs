use crate::circuit_breaker::{CircuitBreaker, State};

#[derive(Debug, Clone, Copy, PartialEq)]
enum MiddleBuffer {
	One(usize),
	Two(usize, usize),
}

#[derive(Debug, PartialEq)]
pub struct Visualizer<'a> {
	cb: &'a mut CircuitBreaker,
	top: Vec<usize>,
	middle: Option<Vec<MiddleBuffer>>,
	bottom: Option<Vec<usize>>,
}

impl<'a> Visualizer<'a> {
	pub fn new(cb: &'a mut CircuitBreaker) -> Self {
		match cb.get_buffer().get_length() {
			0 => panic!("Must have at least one buffer enabled"),
			1 => Self {
				cb,
				top: vec![0],
				middle: None,
				bottom: None,
			},
			2 => Self {
				cb,
				top: vec![0, 1],
				middle: None,
				bottom: None,
			},
			3 => Self {
				cb,
				top: vec![0, 1, 2],
				middle: None,
				bottom: None,
			},
			4 => Self {
				cb,
				top: vec![0, 1, 2],
				middle: None,
				bottom: Some(vec![3]),
			},
			5 => Self {
				cb,
				top: vec![0, 1, 2],
				middle: None,
				bottom: Some(vec![4, 3]),
			},
			6 => Self {
				cb,
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
					cb,
					top: vec![0, 1, 2],
					middle: Some(middle_buffers),
					bottom: Some(bottom),
				}
			},
		}
	}

	fn render_buffer_box_top(&self, index: usize) -> String {
		let is_active = self.cb.get_buffer().get_cursor() == index;
		match is_active {
			true => String::from("┏━━━━━━━━━━━━━━━━━┓"),
			false => String::from("┌─────────────────┐"),
		}
	}

	fn render_buffer_box_middle(&self, index: usize) -> String {
		let is_active = self.cb.get_buffer().get_cursor() == index;
		let infos = self.cb.get_buffer().get_node_info(index);
		match is_active {
			true => format!(
				"┃ B{index:<2} \x1b[42m {:0>3} \x1b[0m \x1b[41m {:0>3} \x1b[0m ┃",
				infos.total_count - infos.failure_count,
				infos.failure_count
			),
			false => format!(
				"│ B{index:<2} \x1b[42m {:0>3} \x1b[0m \x1b[41m {:0>3} \x1b[0m │",
				infos.total_count - infos.failure_count,
				infos.failure_count
			),
		}
	}

	fn render_buffer_box_bottom(&self, index: usize) -> String {
		let is_active = self.cb.get_buffer().get_cursor() == index;
		match is_active {
			true => String::from("┗━━━━━━━━━━━━━━━━━┛"),
			false => String::from("└─────────────────┘"),
		}
	}

	pub fn render(&mut self) -> String {
		let mut output = String::new();

		// NETWORK
		output.push_str(
			r#"
                       ┌─────────────┐
                       │   Service   │
                       └─────────────┘
                              │"#,
		);
		output.push_str(&format!("\n                           Success"));
		output.push_str("\n                              │");
		let state = self.cb.get_state();
		match state {
			State::Closed => output.push_str("\n                              │"),
			State::HalfOpen => output.push_str("\n                              /"),
			State::Open(_) => output.push_str("\n                              -"),
		}
		output.push_str(
			r#"
                              │
                              ▼"#,
		);
		output.push_str(&format!("\n                         Status: {state:?}"));
		output.push_str(&format!("\n                     Error Rate: {}%\n", self.cb.get_error_rate()));

		// RING BUFFER
		let mut top = [String::new(), String::new(), String::new()];
		let mut middle = vec![String::new(), String::new()];
		let mut bottom = [String::new(), String::new(), String::new()];

		// TOP
		for index in 0..self.top.len() {
			top[0].push_str(&self.render_buffer_box_top(index));
			top[1].push_str(&self.render_buffer_box_middle(index));
			top[2].push_str(&self.render_buffer_box_bottom(index));
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
							middle[i + 1].push_str(&format!(
								"         │                                {}",
								self.render_buffer_box_top(*index1)
							));
							middle[i + 2].push_str(&format!(
								"         │                                {}",
								self.render_buffer_box_middle(*index1)
							));
							middle[i + 3].push_str(&format!(
								"         │                                {}",
								self.render_buffer_box_bottom(*index1)
							));
							middle[i + 4].push_str("         │                                         │");
							middle[i + 5].push_str("         │                                         ▼");
							i += 5;
						},
						MiddleBuffer::Two(index1, index2) => {
							middle[i + 1].push_str(&format!(
								"{}                       {}",
								self.render_buffer_box_top(*index1),
								self.render_buffer_box_top(*index2)
							));
							middle[i + 2].push_str(&format!(
								"{}                       {}",
								self.render_buffer_box_middle(*index1),
								self.render_buffer_box_middle(*index2)
							));
							middle[i + 3].push_str(&format!(
								"{}                       {}",
								self.render_buffer_box_bottom(*index1),
								self.render_buffer_box_bottom(*index2)
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
					bottom[0].push_str(&self.render_buffer_box_top(*index));
					bottom[1].push_str(&self.render_buffer_box_middle(*index));
					bottom[2].push_str(&self.render_buffer_box_bottom(*index));
					if *index != b[b.len() - 1] {
						bottom[0].push_str("  ");
						bottom[1].push_str("◀─");
						bottom[2].push_str("  ");
					}
				}
			},
		}

		output.push_str(&top.join("\n"));
		output.push('\n');
		output.push_str(&middle.join("\n"));
		output.push('\n');
		output.push_str(&bottom.join("\n"));
		output
	}

	pub fn record<T, E>(&mut self, input: Result<T, E>) {
		self.cb.record(input);
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::circuit_breaker::{CircuitBreaker, Settings};

	#[test]
	fn render_buffer_box_test() {
		let mut cb = CircuitBreaker::new(Settings { ..Settings::default() });
		let mut vis = Visualizer::new(&mut cb);
		assert_eq!(vis.render_buffer_box_top(0), String::from("┏━━━━━━━━━━━━━━━━━┓"));
		assert_eq!(vis.render_buffer_box_middle(0), String::from("┃ B0  \x1b[42m 000 \x1b[0m \x1b[41m 000 \x1b[0m ┃"));
		assert_eq!(vis.render_buffer_box_bottom(0), String::from("┗━━━━━━━━━━━━━━━━━┛"));

		assert_eq!(vis.render_buffer_box_top(1), String::from("┌─────────────────┐"));
		assert_eq!(vis.render_buffer_box_middle(1), String::from("│ B1  \x1b[42m 000 \x1b[0m \x1b[41m 000 \x1b[0m │"));
		assert_eq!(vis.render_buffer_box_bottom(1), String::from("└─────────────────┘"));

		vis.record::<(), &str>(Err(""));
		vis.record::<(), ()>(Ok(()));
		vis.record::<(), ()>(Ok(()));
		vis.record::<(), &str>(Err(""));
		vis.record::<(), ()>(Ok(()));

		assert_eq!(vis.render_buffer_box_top(0), String::from("┏━━━━━━━━━━━━━━━━━┓"));
		assert_eq!(vis.render_buffer_box_middle(0), String::from("┃ B0  \x1b[42m 003 \x1b[0m \x1b[41m 002 \x1b[0m ┃"));
		assert_eq!(vis.render_buffer_box_bottom(0), String::from("┗━━━━━━━━━━━━━━━━━┛"));
	}

	#[test]
	fn new_test() {
		let mut cb = CircuitBreaker::new(Settings {
			buffer_size: 1,
			..Settings::default()
		});
		assert_eq!(Visualizer::new(&mut cb).top, vec![0]);
		assert_eq!(Visualizer::new(&mut cb).middle, None);
		assert_eq!(Visualizer::new(&mut cb).bottom, None);

		let mut cb = CircuitBreaker::new(Settings {
			buffer_size: 2,
			..Settings::default()
		});
		assert_eq!(Visualizer::new(&mut cb).top, vec![0, 1]);
		assert_eq!(Visualizer::new(&mut cb).middle, None);
		assert_eq!(Visualizer::new(&mut cb).bottom, None);

		let mut cb = CircuitBreaker::new(Settings {
			buffer_size: 3,
			..Settings::default()
		});
		assert_eq!(Visualizer::new(&mut cb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&mut cb).middle, None);
		assert_eq!(Visualizer::new(&mut cb).bottom, None);

		let mut cb = CircuitBreaker::new(Settings {
			buffer_size: 4,
			..Settings::default()
		});
		assert_eq!(Visualizer::new(&mut cb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&mut cb).middle, None);
		assert_eq!(Visualizer::new(&mut cb).bottom, Some(vec![3]));

		let mut cb = CircuitBreaker::new(Settings {
			buffer_size: 5,
			..Settings::default()
		});
		assert_eq!(Visualizer::new(&mut cb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&mut cb).middle, None);
		assert_eq!(Visualizer::new(&mut cb).bottom, Some(vec![4, 3]));

		let mut cb = CircuitBreaker::new(Settings {
			buffer_size: 6,
			..Settings::default()
		});
		assert_eq!(Visualizer::new(&mut cb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&mut cb).middle, None);
		assert_eq!(Visualizer::new(&mut cb).bottom, Some(vec![5, 4, 3]));

		let mut cb = CircuitBreaker::new(Settings {
			buffer_size: 7,
			..Settings::default()
		});
		assert_eq!(Visualizer::new(&mut cb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&mut cb).middle, Some(vec![MiddleBuffer::One(3)]));
		assert_eq!(Visualizer::new(&mut cb).bottom, Some(vec![6, 5, 4]));

		let mut cb = CircuitBreaker::new(Settings {
			buffer_size: 8,
			..Settings::default()
		});
		assert_eq!(Visualizer::new(&mut cb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&mut cb).middle, Some(vec![MiddleBuffer::Two(7, 3)]));
		assert_eq!(Visualizer::new(&mut cb).bottom, Some(vec![6, 5, 4]));

		let mut cb = CircuitBreaker::new(Settings {
			buffer_size: 9,
			..Settings::default()
		});
		assert_eq!(Visualizer::new(&mut cb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&mut cb).middle, Some(vec![MiddleBuffer::Two(8, 3), MiddleBuffer::One(4),]));
		assert_eq!(Visualizer::new(&mut cb).bottom, Some(vec![7, 6, 5]));

		let mut cb = CircuitBreaker::new(Settings {
			buffer_size: 10,
			..Settings::default()
		});
		assert_eq!(Visualizer::new(&mut cb).top, vec![0, 1, 2]);
		assert_eq!(Visualizer::new(&mut cb).middle, Some(vec![MiddleBuffer::Two(9, 3), MiddleBuffer::Two(8, 4),]));
		assert_eq!(Visualizer::new(&mut cb).bottom, Some(vec![7, 6, 5]));

		let mut cb = CircuitBreaker::new(Settings {
			buffer_size: 11,
			..Settings::default()
		});
		assert_eq!(Visualizer::new(&mut cb).top, vec![0, 1, 2]);
		assert_eq!(
			Visualizer::new(&mut cb).middle,
			Some(vec![MiddleBuffer::Two(10, 3), MiddleBuffer::Two(9, 4), MiddleBuffer::One(5),])
		);
		assert_eq!(Visualizer::new(&mut cb).bottom, Some(vec![8, 7, 6]));

		let mut cb = CircuitBreaker::new(Settings {
			buffer_size: 12,
			..Settings::default()
		});
		assert_eq!(Visualizer::new(&mut cb).top, vec![0, 1, 2]);
		assert_eq!(
			Visualizer::new(&mut cb).middle,
			Some(vec![
				MiddleBuffer::Two(11, 3),
				MiddleBuffer::Two(10, 4),
				MiddleBuffer::Two(9, 5),
			])
		);
		assert_eq!(Visualizer::new(&mut cb).bottom, Some(vec![8, 7, 6]));

		let mut cb = CircuitBreaker::new(Settings {
			buffer_size: 13,
			..Settings::default()
		});
		assert_eq!(Visualizer::new(&mut cb).top, vec![0, 1, 2]);
		assert_eq!(
			Visualizer::new(&mut cb).middle,
			Some(vec![
				MiddleBuffer::Two(12, 3),
				MiddleBuffer::Two(11, 4),
				MiddleBuffer::Two(10, 5),
				MiddleBuffer::One(6),
			])
		);
		assert_eq!(Visualizer::new(&mut cb).bottom, Some(vec![9, 8, 7]));
	}
}
