mod cli;
mod commands;
mod error;

use clap::Parser;

fn main() {
	let args = cli::Cli::parse();

	let command_result = match args.command {
		cli::Commands::Cat { filename, delimiter, no_header } => {
			commands::cat::run(&filename, &delimiter, no_header)
		}
	};

	match command_result {
		Ok(_) => (),
		Err(e) => {
			eprintln!("{e}");
		}
	}
}