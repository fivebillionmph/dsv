mod cli;
mod app;
mod error;
mod fields_subset;
mod read_iterator;

use std::io::Write as _;

use anyhow::Result as Res;

fn main() {
	let app_result = run();

	match app_result {
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

fn run() -> Res<()> {
	let (filename, delimiter, run_options) = cli::get_run_options()?;
	app::run(&filename, &delimiter, &run_options)?;

	Ok(())
}