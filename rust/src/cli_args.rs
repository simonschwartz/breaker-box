use std::time::Duration;

use crate::{circuit_breaker::Settings, cli_helpers::exit_with_error};

pub fn parse_args(args: Vec<String>) -> Settings {
	let mut settings: Settings = Default::default();

	let mut args_iter = args.into_iter();
	while let Some(arg) = args_iter.next() {
		match arg.as_str() {
			"-b" | "--buffer_size" => {
				settings.buffer_size = args_iter
					.next()
					.unwrap_or_else(|| exit_with_error("The buffer_size flag requires an additional argument", 1))
					.parse()
					.unwrap_or_else(|_| exit_with_error("The buffer_size argument must be a number", 1));
			},
			"-m" | "--min_eval_size" => {
				settings.min_eval_size = args_iter
					.next()
					.unwrap_or_else(|| exit_with_error("The min_eval_size flag requires an additional argument", 1))
					.parse()
					.unwrap_or_else(|_| exit_with_error("The min_eval_size argument must be a number", 1));
			},
			"-e" | "--error_threshold" => {
				settings.error_threshold = args_iter
					.next()
					.unwrap_or_else(|| exit_with_error("The error_threshold flag requires an additional argument", 1))
					.parse()
					.unwrap_or_else(|_| exit_with_error("The error_threshold argument must be a number", 1));
			},
			"-r" | "--retry_timeout" => {
				let duration = args_iter
					.next()
					.unwrap_or_else(|| exit_with_error("The retry_timeout flag requires an additional argument", 1))
					.parse()
					.unwrap_or_else(|_| exit_with_error("The retry_timeout argument must be a number", 1));
				settings.retry_timeout = Duration::from_secs(duration);
			},
			"-s" | "--buffer_span_duration" => {
				let duration = args_iter
					.next()
					.unwrap_or_else(|| exit_with_error("The buffer_span_duration flag requires an additional argument", 1))
					.parse()
					.unwrap_or_else(|_| exit_with_error("The buffer_span_duration argument must be a number", 1));
				settings.buffer_span_duration = Duration::from_secs(duration);
			},
			"-t" | "--trial_success_required" => {
				settings.trial_success_required = args_iter
					.next()
					.unwrap_or_else(|| exit_with_error("The trial_success_required flag requires an additional argument", 1))
					.parse()
					.unwrap_or_else(|_| exit_with_error("The trial_success_required argument must be a number", 1));
			},
			_ => {},
		}
	}
	settings
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_args_long_flags() {
		assert_eq!(
			parse_args(vec![
				String::from("--buffer_size"),
				String::from("42"),
				String::from("--min_eval_size"),
				String::from("11"),
				String::from("--error_threshold"),
				String::from("10.78"),
				String::from("--retry_timeout"),
				String::from("200"),
				String::from("--buffer_span_duration"),
				String::from("550"),
				String::from("--trial_success_required"),
				String::from("666"),
				String::from("--unknown"),
			]),
			Settings {
				buffer_size: 42,
				min_eval_size: 11,
				error_threshold: 10.78,
				retry_timeout: Duration::from_secs(200),
				buffer_span_duration: Duration::from_secs(550),
				trial_success_required: 666,
			}
		);
	}

	#[test]
	fn parse_args_short_flags() {
		assert_eq!(
			parse_args(vec![
				String::from("-b"),
				String::from("0"),
				String::from("-m"),
				String::from("875"),
				String::from("-e"),
				String::from("5647.1"),
				String::from("-r"),
				String::from("62"),
				String::from("-s"),
				String::from("279"),
				String::from("-t"),
				String::from("0"),
				String::from("-x"),
			]),
			Settings {
				buffer_size: 0,
				min_eval_size: 875,
				error_threshold: 5647.1,
				retry_timeout: Duration::from_secs(62),
				buffer_span_duration: Duration::from_secs(279),
				trial_success_required: 0,
			}
		);
	}

	#[test]
	fn parse_args_buffer_size() {
		assert_eq!(
			parse_args(vec![String::from("--buffer_size"), String::from("10")]),
			Settings {
				buffer_size: 10,
				..Default::default()
			}
		);
		assert_eq!(
			parse_args(vec![String::from("-b"), String::from("0")]),
			Settings {
				buffer_size: 0,
				..Default::default()
			}
		);
		assert_eq!(
			parse_args(vec![String::from("-b"), String::from("999")]),
			Settings {
				buffer_size: 999,
				..Default::default()
			}
		);
	}

	#[test]
	#[should_panic]
	fn parse_args_buffer_size_error_negative() {
		parse_args(vec![String::from("-b"), String::from("-9")]);
	}

	#[test]
	#[should_panic]
	fn parse_args_buffer_size_error_missing() {
		parse_args(vec![String::from("-b")]);
	}

	#[test]
	#[should_panic]
	fn parse_args_buffer_size_error_missing2() {
		parse_args(vec![String::from("-b"), String::from("-b")]);
	}

	#[test]
	fn parse_args_min_eval_size() {
		assert_eq!(
			parse_args(vec![String::from("--min_eval_size"), String::from("10")]),
			Settings {
				min_eval_size: 10,
				..Default::default()
			}
		);
		assert_eq!(
			parse_args(vec![String::from("-m"), String::from("0")]),
			Settings {
				min_eval_size: 0,
				..Default::default()
			}
		);
		assert_eq!(
			parse_args(vec![String::from("-m"), String::from("999")]),
			Settings {
				min_eval_size: 999,
				..Default::default()
			}
		);
	}

	#[test]
	#[should_panic]
	fn parse_args_min_eval_size_error_negative() {
		parse_args(vec![String::from("-m"), String::from("-9")]);
	}

	#[test]
	#[should_panic]
	fn parse_args_min_eval_size_error_missing() {
		parse_args(vec![String::from("-m")]);
	}

	#[test]
	#[should_panic]
	fn parse_args_min_eval_size_error_missing2() {
		parse_args(vec![String::from("-m"), String::from("-m")]);
	}

	#[test]
	fn parse_args_error_threshold() {
		assert_eq!(
			parse_args(vec![String::from("--error_threshold"), String::from("10")]),
			Settings {
				error_threshold: 10.0,
				..Default::default()
			}
		);
		assert_eq!(
			parse_args(vec![String::from("-e"), String::from("0")]),
			Settings {
				error_threshold: 0.0,
				..Default::default()
			}
		);
		assert_eq!(
			parse_args(vec![String::from("-e"), String::from("999.9")]),
			Settings {
				error_threshold: 999.9,
				..Default::default()
			}
		);
	}

	#[test]
	#[should_panic]
	fn parse_args_error_threshold_error_missing() {
		parse_args(vec![String::from("-e")]);
	}

	#[test]
	#[should_panic]
	fn parse_args_error_threshold_error_missing2() {
		parse_args(vec![String::from("-e"), String::from("-e")]);
	}

	#[test]
	fn parse_args_retry_timeout() {
		assert_eq!(
			parse_args(vec![String::from("--retry_timeout"), String::from("10")]),
			Settings {
				retry_timeout: Duration::from_secs(10),
				..Default::default()
			}
		);
		assert_eq!(
			parse_args(vec![String::from("-r"), String::from("0")]),
			Settings {
				retry_timeout: Duration::from_secs(0),
				..Default::default()
			}
		);
		assert_eq!(
			parse_args(vec![String::from("-r"), String::from("999")]),
			Settings {
				retry_timeout: Duration::from_secs(999),
				..Default::default()
			}
		);
	}

	#[test]
	#[should_panic]
	fn parse_args_retry_timeout_error_negative() {
		parse_args(vec![String::from("-r"), String::from("-9")]);
	}

	#[test]
	#[should_panic]
	fn parse_args_retry_timeout_error_missing() {
		parse_args(vec![String::from("-r")]);
	}

	#[test]
	#[should_panic]
	fn parse_args_retry_timeout_error_missing2() {
		parse_args(vec![String::from("-r"), String::from("-r")]);
	}

	#[test]
	fn parse_args_buffer_span_duration() {
		assert_eq!(
			parse_args(vec![String::from("--buffer_span_duration"), String::from("10")]),
			Settings {
				buffer_span_duration: Duration::from_secs(10),
				..Default::default()
			}
		);
		assert_eq!(
			parse_args(vec![String::from("-s"), String::from("0")]),
			Settings {
				buffer_span_duration: Duration::from_secs(0),
				..Default::default()
			}
		);
		assert_eq!(
			parse_args(vec![String::from("-s"), String::from("999")]),
			Settings {
				buffer_span_duration: Duration::from_secs(999),
				..Default::default()
			}
		);
	}

	#[test]
	#[should_panic]
	fn parse_args_buffer_span_duration_error_negative() {
		parse_args(vec![String::from("-s"), String::from("-9")]);
	}

	#[test]
	#[should_panic]
	fn parse_args_buffer_span_duration_error_missing() {
		parse_args(vec![String::from("-s")]);
	}

	#[test]
	#[should_panic]
	fn parse_args_buffer_span_duration_error_missing2() {
		parse_args(vec![String::from("-s"), String::from("-s")]);
	}

	#[test]
	fn parse_args_trial_success_required() {
		assert_eq!(
			parse_args(vec![String::from("--trial_success_required"), String::from("10")]),
			Settings {
				trial_success_required: 10,
				..Default::default()
			}
		);
		assert_eq!(
			parse_args(vec![String::from("-t"), String::from("0")]),
			Settings {
				trial_success_required: 0,
				..Default::default()
			}
		);
		assert_eq!(
			parse_args(vec![String::from("-t"), String::from("999")]),
			Settings {
				trial_success_required: 999,
				..Default::default()
			}
		);
	}

	#[test]
	#[should_panic]
	fn parse_args_trial_success_required_error_negative() {
		parse_args(vec![String::from("-t"), String::from("-9")]);
	}

	#[test]
	#[should_panic]
	fn parse_args_trial_success_required_error_missing() {
		parse_args(vec![String::from("-t")]);
	}

	#[test]
	#[should_panic]
	fn parse_args_trial_success_required_error_missing2() {
		parse_args(vec![String::from("-t"), String::from("-t")]);
	}
}
