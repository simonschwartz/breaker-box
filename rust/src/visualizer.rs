use std::{
	io::{self, Read},
	process::Command,
	sync::mpsc,
	thread,
	time::{Duration, Instant},
};

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
		match cb.get_buffer().get_buffer_size() {
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

	fn render_buffer_box_top(&mut self, index: usize) -> String {
		let buffer_span_duration = self.cb.get_settings().buffer_span_duration;
		let is_active = if self.cb.get_state() == State::Closed {
			self.cb.get_buffer().get_cursor(buffer_span_duration, Instant::now()) == index
		} else {
			false
		};
		match is_active {
			true => String::from("┏━━━━━━━━━━━━━━━━━┓"),
			false => String::from("┌─────────────────┐"),
		}
	}

	fn render_buffer_box_middle(&mut self, index: usize) -> String {
		let buffer_span_duration = self.cb.get_settings().buffer_span_duration;
		let is_active = if self.cb.get_state() == State::Closed {
			self.cb.get_buffer().get_cursor(buffer_span_duration, Instant::now()) == index
		} else {
			false
		};
		let infos = self.cb.get_buffer().get_node_info(index);
		match is_active {
			true => format!(
				"┃ B{index:<2} \x1b[42m {:0>3} \x1b[0m \x1b[41m {:0>3} \x1b[0m ┃",
				infos.success_count, infos.failure_count
			),
			false => format!(
				"│ B{index:<2} \x1b[42m {:0>3} \x1b[0m \x1b[41m {:0>3} \x1b[0m │",
				infos.success_count, infos.failure_count
			),
		}
	}

	fn render_buffer_box_bottom(&mut self, index: usize) -> String {
		let buffer_span_duration = self.cb.get_settings().buffer_span_duration;
		let is_active = if self.cb.get_state() == State::Closed {
			self.cb.get_buffer().get_cursor(buffer_span_duration, Instant::now()) == index
		} else {
			false
		};
		match is_active {
			true => String::from("┗━━━━━━━━━━━━━━━━━┛"),
			false => String::from("└─────────────────┘"),
		}
	}

	pub fn record<T, E>(&mut self, input: Result<T, E>) {
		self.cb.record(input);
	}

	pub fn render<T, E>(&mut self, input: Option<Result<T, E>>) -> String {
		let mut output = String::new();
		let mut request_color = "";
		let mut request = "   │   ";
		if let Some(i) = input {
			if i.is_ok() {
				request_color = "\x1b[32m";
				request = "Success";
			} else {
				request_color = "\x1b[31m";
				request = "Failure";
			}
		}

		// NETWORK
		let state = self.cb.get_state();
		output.push_str(
			r#"
                       ┌─────────────┐
                       │   Service   │
                       └─────────────┘"#,
		);
		output.push_str(&format!("\n                              {request_color}│"));
		output.push_str(&format!("\n                           {request}"));
		output.push_str("\n                              │");
		output.push_str(&format!("\n                              {state:#}"));
		output.push_str("\n                              │");
		output.push_str("\n                              ▼\x1b[0m");
		output.push_str(&format!("\n                         Status: {state}"));
		output.push_str(&format!("\n                     Error Rate: {:0<6?}%\n", self.cb.get_error_rate()));
		match state {
			State::Closed => {
				let buffer_span_duration = self.cb.get_settings().buffer_span_duration;
				let timer = self
					.cb
					.get_settings()
					.buffer_span_duration
					.saturating_sub(self.cb.get_buffer().get_elapsed_time(buffer_span_duration, Instant::now()));
				output.push_str(&format!("                    Next Buffer: {}s   \n", timer.as_secs()));
			},
			State::Open(duration) => {
				let timer = self.cb.get_settings().retry_timeout.saturating_sub(duration.elapsed());
				output.push_str(&format!("                          Retry: {}s   \n", timer.as_secs()));
			},
			State::HalfOpen => {
				output.push_str(&format!(
					"                  Trial Success: {}/{}   \n",
					self.cb.get_trial_success(),
					self.cb.get_settings().trial_success_required
				));
			},
		}

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
		match self.middle.clone() {
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
								.push_str(&format!("         │                                {}", self.render_buffer_box_top(index1)));
							middle[i + 2].push_str(&format!(
								"         │                                {}",
								self.render_buffer_box_middle(index1)
							));
							middle[i + 3].push_str(&format!(
								"         │                                {}",
								self.render_buffer_box_bottom(index1)
							));
							middle[i + 4].push_str("         │                                         │");
							middle[i + 5].push_str("         │                                         ▼");
							i += 5;
						},
						MiddleBuffer::Two(index1, index2) => {
							middle[i + 1].push_str(&format!(
								"{}                       {}",
								self.render_buffer_box_top(index1),
								self.render_buffer_box_top(index2)
							));
							middle[i + 2].push_str(&format!(
								"{}                       {}",
								self.render_buffer_box_middle(index1),
								self.render_buffer_box_middle(index2)
							));
							middle[i + 3].push_str(&format!(
								"{}                       {}",
								self.render_buffer_box_bottom(index1),
								self.render_buffer_box_bottom(index2)
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
		match self.bottom.clone() {
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

				for index in &b {
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
		output.push('\n');
		output.push_str("\n\n    [s]=Successful request  [f]=Request Failure  [q]=Quit\n");
		output
	}

	pub fn start(&mut self, periodically: bool) -> io::Result<()> {
		#[cfg(target_os = "windows")]
		compile_error!(
			"Windows is not supported for the visualizer due to the lack of raw mode. Use WSL to make it compile!"
		);

		let _raw = RawMode::enter()?;

		// A thread just for stdin
		let (sender, receiver) = mpsc::channel::<u8>();
		{
			let stdin = io::stdin();
			thread::spawn(move || {
				let mut lock = stdin.lock();
				let mut buffer = [0u8; 1];
				while lock.read_exact(&mut buffer).is_ok() {
					if sender.send(buffer[0]).is_err() {
						break;
					}
				}
			});
		}

		let mut last_tick = Instant::now();
		let render = self.render::<(), &str>(None);
		let lines = render.bytes().filter(|&b| b == b'\n').count();
		let reset_pos = format!("\x1b[{lines}F");
		print!("{render}");

		loop {
			if let Ok(byte) = receiver.try_recv() {
				match byte as char {
					'q' => {
						println!("Bye...");
						break;
					},
					's' => {
						self.record::<(), &str>(Ok(()));
						print!("{reset_pos}{}", self.render::<(), &str>(Some(Ok(()))));
						last_tick = Instant::now();
					},
					'f' => {
						self.record::<(), &str>(Err(""));
						print!("{reset_pos}{}", self.render::<(), &str>(Some(Err(""))));
						last_tick = Instant::now();
					},
					'x' => {
						// Debug output and quit
						println!(
							"\n ╔╦╗ ╔═╗ ╔╗  ╦ ╦ ╔═╗\n  ║║ ║╣  ╠╩╗ ║ ║ ║ ╦\n ═╩╝ ╚═╝ ╚═╝ ╚═╝ ╚═╝\n\n{:#?}",
							self.cb.get_buffer()
						);
						break;
					},
					_ => {},
				}
			}

			if periodically && last_tick.elapsed() >= Duration::from_secs(1) {
				print!("{reset_pos}{}", self.render::<(), &str>(None));
				last_tick = Instant::now();
			}
		}

		Ok(())
	}
}

struct RawMode;

impl RawMode {
	fn enter() -> io::Result<Self> {
		Command::new("stty").arg("-icanon").arg("-echo").spawn()?.wait()?;
		Ok(RawMode)
	}
}

impl Drop for RawMode {
	fn drop(&mut self) {
		let _ = Command::new("stty").arg("icanon").arg("echo").spawn().and_then(|mut c| c.wait());
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

	#[test]
	fn end_2_end_test() {
		let buffer_span_duration = Duration::from_secs(1);
		let mut cb = CircuitBreaker::new(Settings {
			buffer_span_duration,
			..Settings::default()
		});
		let vis = Visualizer::new(&mut cb);

		assert_eq!(vis.cb.get_buffer().get_cursor(buffer_span_duration, Instant::now()), 0);
		vis.cb.record::<(), &str>(Ok(()));
		vis.cb.record::<(), &str>(Ok(()));
		vis.cb.record::<(), &str>(Ok(()));
		std::thread::sleep(buffer_span_duration);
		assert_eq!(vis.cb.get_buffer().get_cursor(buffer_span_duration, Instant::now()), 1);
		assert_eq!(vis.cb.get_buffer().get_node_info(0).success_count, 3);
		assert_eq!(vis.cb.get_buffer().get_node_info(1).success_count, 0);
		assert_eq!(vis.cb.get_buffer().get_node_info(2).success_count, 0);
		assert_eq!(vis.cb.get_buffer().get_node_info(3).success_count, 0);
		assert_eq!(vis.cb.get_buffer().get_node_info(4).success_count, 0);
		vis.cb.record::<(), &str>(Ok(()));
		vis.cb.record::<(), &str>(Ok(()));
		vis.cb.record::<(), &str>(Ok(()));
		std::thread::sleep(buffer_span_duration);
		assert_eq!(vis.cb.get_buffer().get_cursor(buffer_span_duration, Instant::now()), 2);
		assert_eq!(vis.cb.get_buffer().get_node_info(0).success_count, 3);
		assert_eq!(vis.cb.get_buffer().get_node_info(1).success_count, 3);
		assert_eq!(vis.cb.get_buffer().get_node_info(2).success_count, 0);
		assert_eq!(vis.cb.get_buffer().get_node_info(3).success_count, 0);
		assert_eq!(vis.cb.get_buffer().get_node_info(4).success_count, 0);
		vis.cb.record::<(), &str>(Ok(()));
		vis.cb.record::<(), &str>(Ok(()));
		vis.cb.record::<(), &str>(Ok(()));
		std::thread::sleep(buffer_span_duration);
		assert_eq!(vis.cb.get_buffer().get_cursor(buffer_span_duration, Instant::now()), 3);
		assert_eq!(vis.cb.get_buffer().get_node_info(0).success_count, 3);
		assert_eq!(vis.cb.get_buffer().get_node_info(1).success_count, 3);
		assert_eq!(vis.cb.get_buffer().get_node_info(2).success_count, 3);
		assert_eq!(vis.cb.get_buffer().get_node_info(3).success_count, 0);
		assert_eq!(vis.cb.get_buffer().get_node_info(4).success_count, 0);
		vis.cb.record::<(), &str>(Ok(()));
		vis.cb.record::<(), &str>(Ok(()));
		vis.cb.record::<(), &str>(Ok(()));
		std::thread::sleep(buffer_span_duration);
		assert_eq!(vis.cb.get_buffer().get_cursor(buffer_span_duration, Instant::now()), 4);
		assert_eq!(vis.cb.get_buffer().get_node_info(0).success_count, 3);
		assert_eq!(vis.cb.get_buffer().get_node_info(1).success_count, 3);
		assert_eq!(vis.cb.get_buffer().get_node_info(2).success_count, 3);
		assert_eq!(vis.cb.get_buffer().get_node_info(3).success_count, 3);
		assert_eq!(vis.cb.get_buffer().get_node_info(4).success_count, 0);
		vis.cb.record::<(), &str>(Ok(()));
		vis.cb.record::<(), &str>(Ok(()));
		vis.cb.record::<(), &str>(Ok(()));
		std::thread::sleep(buffer_span_duration);
		assert_eq!(vis.cb.get_buffer().get_cursor(buffer_span_duration, Instant::now()), 0);
		assert_eq!(vis.cb.get_buffer().get_node_info(0).success_count, 0);
		assert_eq!(vis.cb.get_buffer().get_node_info(1).success_count, 3);
		assert_eq!(vis.cb.get_buffer().get_node_info(2).success_count, 3);
		assert_eq!(vis.cb.get_buffer().get_node_info(3).success_count, 3);
		assert_eq!(vis.cb.get_buffer().get_node_info(4).success_count, 3);
		vis.cb.record::<(), &str>(Ok(()));

		// We skip 3 nodes ahead
		std::thread::sleep(buffer_span_duration + buffer_span_duration + buffer_span_duration);
		vis.cb.evaluate_state();

		assert_eq!(vis.cb.get_buffer().get_node_info(0).success_count, 1);
		assert_eq!(vis.cb.get_buffer().get_node_info(1).success_count, 0); // skipped
		assert_eq!(vis.cb.get_buffer().get_node_info(2).success_count, 0); // skipped
		assert_eq!(vis.cb.get_buffer().get_node_info(3).success_count, 0); // current
		assert_eq!(vis.cb.get_buffer().get_node_info(4).success_count, 3);
		assert_eq!(vis.cb.get_buffer().get_cursor(buffer_span_duration, Instant::now()), 3);
	}
}
