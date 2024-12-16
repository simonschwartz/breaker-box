pub fn exit_with_error(error: &str, code: i32) -> ! {
	eprintln!("{error}");

	if cfg!(test) {
		panic!("error=\"{error}\" code=\"{code}\"");
	} else {
		std::process::exit(code);
	}
}

pub fn help() {
	println!(
		r#"
Usage: circuitbreaker [OPTIONS]

Options:
  -b, --buffer_size SIZE              Specify the capacity of the ring buffer.
  -m, --min_eval_size NUMBER          Define the minimum number of events required in the buffer to evaluate the error rate.
  -e, --error_threshold FLOAT         Set the error rate percentage that will trigger the circuit to open.
  -r, --retry_timeout SECONDS         Specify the duration (in seconds) the circuit breaker remains open before transitioning to half-open.
  -s, --buffer_span_duration SECONDS  Determine the duration (in seconds) each node/span in the buffer stores data.
  -t, --trial_success_required NUMBER Set the number of consecutive successes required to close a half-open circuit.
  -h, --help                          Display this help message and exit.
  -v, --version                       Display version information and exit.
	"#
	);
}
