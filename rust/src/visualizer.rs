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
struct Grid {
	top: Vec<Buffer>,
	middle: Option<Vec<MiddleBuffer>>,
	bottom: Option<Vec<Buffer>>,
}

impl Grid {
	pub fn new(buffer: usize) -> Self {
		match buffer {
			0 => panic!("Must have at least one buffer enabled"),
			1 => Self {
				top: vec![Buffer::Index(0)],
				middle: None,
				bottom: None,
			},
			2 => Self {
				top: vec![Buffer::Index(0), Buffer::Index(1)],
				middle: None,
				bottom: None,
			},
			3 => Self {
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: None,
			},
			4 => Self {
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: Some(vec![Buffer::Index(3)]),
			},
			5 => Self {
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: Some(vec![Buffer::Index(3), Buffer::Index(4)]),
			},
			6 => Self {
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: Some(vec![Buffer::Index(3), Buffer::Index(4), Buffer::Index(5)]),
			},
			b => {
				let n = (b - 7) / 2;
				let mut middle = Vec::new();
				for index in (0..n).step_by(2) {
					middle.push(MiddleBuffer::Two(Buffer::Index(index), Buffer::Index(index + 1)));
				}
				let rest = b - n - 6;
				middle.push(if b % 2 == 0 {
					MiddleBuffer::Two(Buffer::Index(rest), Buffer::Index(rest + 1))
				} else {
					MiddleBuffer::One(Buffer::Index(rest))
				});

				Self {
					top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
					middle: Some(middle),
					bottom: Some(vec![Buffer::Index(b - 2), Buffer::Index(b - 1), Buffer::Index(b)]),
				}
			},
		}
	}

	pub fn render(&self) -> String {
		todo!()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn new_test() {
		assert_eq!(
			Grid::new(1),
			Grid {
				top: vec![Buffer::Index(0)],
				middle: None,
				bottom: None,
			}
		);
		assert_eq!(
			Grid::new(2),
			Grid {
				top: vec![Buffer::Index(0), Buffer::Index(1)],
				middle: None,
				bottom: None,
			}
		);
		assert_eq!(
			Grid::new(3),
			Grid {
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: None,
			}
		);
		assert_eq!(
			Grid::new(4),
			Grid {
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: Some(vec![Buffer::Index(3)]),
			}
		);
		assert_eq!(
			Grid::new(5),
			Grid {
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: Some(vec![Buffer::Index(3), Buffer::Index(4)]),
			}
		);
		assert_eq!(
			Grid::new(6),
			Grid {
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: None,
				bottom: Some(vec![Buffer::Index(3), Buffer::Index(4), Buffer::Index(5)]),
			}
		);
		assert_eq!(
			Grid::new(7),
			Grid {
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: Some(vec![MiddleBuffer::One(Buffer::Index(3))]),
				bottom: Some(vec![Buffer::Index(4), Buffer::Index(5), Buffer::Index(6)]),
			}
		);
		assert_eq!(
			Grid::new(8),
			Grid {
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: Some(vec![MiddleBuffer::Two(Buffer::Index(3), Buffer::Index(4))]),
				bottom: Some(vec![Buffer::Index(5), Buffer::Index(6), Buffer::Index(7)]),
			}
		);
		assert_eq!(
			Grid::new(9),
			Grid {
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: Some(vec![
					MiddleBuffer::Two(Buffer::Index(3), Buffer::Index(4)),
					MiddleBuffer::One(Buffer::Index(5)),
				]),
				bottom: Some(vec![Buffer::Index(6), Buffer::Index(7), Buffer::Index(8)]),
			}
		);
		assert_eq!(
			Grid::new(9),
			Grid {
				top: vec![Buffer::Index(0), Buffer::Index(1), Buffer::Index(2)],
				middle: Some(vec![
					MiddleBuffer::Two(Buffer::Index(3), Buffer::Index(4)),
					MiddleBuffer::Two(Buffer::Index(5), Buffer::Index(6)),
				]),
				bottom: Some(vec![Buffer::Index(7), Buffer::Index(8), Buffer::Index(9)]),
			}
		);
		assert_eq!(
			Grid::new(10),
			Grid {
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
			Grid::new(11),
			Grid {
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
			Grid::new(12),
			Grid {
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
			Grid::new(13),
			Grid {
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
