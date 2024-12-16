#[derive(Debug, Clone, Copy, PartialEq)]
enum Row {
	Half,
	Full,
}

#[derive(Debug, PartialEq)]
struct Grid {
	top: usize,
	middle: Option<Vec<Row>>,
	bottom: usize,
}

impl Grid {
	pub fn new(buffer: usize) -> Self {
		match buffer {
			0 => Self {
				top: 0,
				middle: None,
				bottom: 0,
			},
			1 => Self {
				top: 1,
				middle: None,
				bottom: 0,
			},
			2 => Self {
				top: 2,
				middle: None,
				bottom: 0,
			},
			3 => Self {
				top: 3,
				middle: None,
				bottom: 0,
			},
			4 => Self {
				top: 3,
				middle: None,
				bottom: 1,
			},
			5 => Self {
				top: 3,
				middle: None,
				bottom: 2,
			},
			6 => Self {
				top: 3,
				middle: None,
				bottom: 3,
			},
			b => {
				let n = ((b - 7) / 2);
				let mut middle = vec![Row::Full; n];
				middle.push(if b % 2 == 0 { Row::Full } else { Row::Half });
				Self {
					top: 3,
					middle: Some(middle),
					bottom: 3,
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
			Grid::new(0),
			Grid {
				top: 0,
				middle: None,
				bottom: 0,
			}
		);
		assert_eq!(
			Grid::new(1),
			Grid {
				top: 1,
				middle: None,
				bottom: 0,
			}
		);
		assert_eq!(
			Grid::new(2),
			Grid {
				top: 2,
				middle: None,
				bottom: 0,
			}
		);
		assert_eq!(
			Grid::new(3),
			Grid {
				top: 3,
				middle: None,
				bottom: 0,
			}
		);
		assert_eq!(
			Grid::new(4),
			Grid {
				top: 3,
				middle: None,
				bottom: 1,
			}
		);
		assert_eq!(
			Grid::new(5),
			Grid {
				top: 3,
				middle: None,
				bottom: 2,
			}
		);
		assert_eq!(
			Grid::new(6),
			Grid {
				top: 3,
				middle: None,
				bottom: 3,
			}
		);
		assert_eq!(
			Grid::new(7),
			Grid {
				top: 3,
				middle: Some(vec![Row::Half]),
				bottom: 3,
			}
		);
		assert_eq!(
			Grid::new(8),
			Grid {
				top: 3,
				middle: Some(vec![Row::Full]),
				bottom: 3,
			}
		);
		assert_eq!(
			Grid::new(9),
			Grid {
				top: 3,
				middle: Some(vec![Row::Full, Row::Half]),
				bottom: 3,
			}
		);
		assert_eq!(
			Grid::new(10),
			Grid {
				top: 3,
				middle: Some(vec![Row::Full, Row::Full]),
				bottom: 3,
			}
		);
		assert_eq!(
			Grid::new(11),
			Grid {
				top: 3,
				middle: Some(vec![Row::Full, Row::Full, Row::Half]),
				bottom: 3,
			}
		);
		assert_eq!(
			Grid::new(12),
			Grid {
				top: 3,
				middle: Some(vec![Row::Full, Row::Full, Row::Full]),
				bottom: 3,
			}
		);
		assert_eq!(
			Grid::new(13),
			Grid {
				top: 3,
				middle: Some(vec![Row::Full, Row::Full, Row::Full, Row::Half]),
				bottom: 3,
			}
		);
	}
}
