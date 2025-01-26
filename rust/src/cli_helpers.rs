pub fn exit_with_error(error: &str, code: i32) -> ! {
	eprintln!("{error}");

	if cfg!(test) {
		panic!("error=\"{error}\" code=\"{code}\"");
	} else {
		std::process::exit(code);
	}
}

pub fn help() -> String {
	r#"
Usage: circuitbreaker [OPTIONS]

Options:
  -b, --buffer_size            SIZE    Specify the capacity of the ring buffer.
  -m, --min_eval_size          NUMBER  Define the minimum number of events
                                       required in the buffer to evaluate the
                                       error rate.
  -e, --error_threshold        FLOAT   Set the error rate percentage that will
                                       trigger the circuit to open.
  -r, --retry_timeout          SECONDS Specify the duration (in seconds) the
                                       circuit breaker remains open before
                                       transitioning to half-open.
  -s, --buffer_span_duration   SECONDS Determine the duration (in seconds) each
                                       node/span in the buffer stores data.
  -t, --trial_success_required NUMBER  Set the number of consecutive successes
                                       required to close a half-open circuit.
  -a, --noautoplay                     Don't auto-play the visualizer and
                                       refresh every second.
  -h, --help                           Display this help message and exit.
  -v, --version                        Display version information and exit.
	"#
	.to_string()
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::circuit_breaker::Settings;

	#[test]
	fn help_test() {
		let settings = Settings::default();
		let debug_output = format!("{:#?}", settings);

		let field_names: Vec<_> = debug_output
			.lines()
			.skip(1)
			.filter_map(|line| line.trim().split_once(":").map(|(field, _)| field.trim()))
			.collect();

		let help = help();
		for field in &field_names {
			assert!(help.contains(field), "Field name '{}' not found in help", field);
		}
	}
}
