mod cli;
mod commands;
mod error;

use std::io::Write as _;

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
			let mut silent_error = false;
			if let Some(et) = e.downcast_ref::<std::io::Error>() {
				match et.kind() {
					std::io::ErrorKind::BrokenPipe => {
						silent_error = true;
					},
					_ => (),
				}
			}

			if !silent_error {
				let _ = writeln!(&mut std::io::stderr(), "{}", e);
			}
		}
	}
}