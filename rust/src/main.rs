mod circuit_breaker;
mod cli;
mod ring_buffer;

use std::env;

fn main() {
	let args: Vec<String> = env::args().skip(1).collect();

	if args.contains(&String::from("-h")) || args.contains(&String::from("--help")) {
		cli::help();
		return;
	}

	if args.contains(&String::from("-v"))
		|| args.contains(&String::from("-V"))
		|| args.contains(&String::from("--version"))
	{
		println!("v{}", env!("CARGO_PKG_VERSION"));
		return;
	}

	let settings = cli::parse_args(args);
	let _ = circuit_breaker::CircuitBreaker::new(settings);
}
