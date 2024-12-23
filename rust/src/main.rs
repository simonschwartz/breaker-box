mod circuit_breaker;
mod cli_args;
mod cli_helpers;
mod ring_buffer;
mod visualizer;

use std::env;

fn main() {
	let args: Vec<String> = env::args().skip(1).collect();

	if args.contains(&String::from("-h")) || args.contains(&String::from("--help")) {
		cli_helpers::help();
		return;
	}

	if args.contains(&String::from("-v"))
		|| args.contains(&String::from("-V"))
		|| args.contains(&String::from("--version"))
	{
		println!("v{}", env!("CARGO_PKG_VERSION"));
		return;
	}

	let settings = cli_args::parse_args(args);
	let mut cb = circuit_breaker::CircuitBreaker::new(settings);

	let vis = visualizer::Visualizer::new(&mut cb);
	println!("{}", vis.render());
}
