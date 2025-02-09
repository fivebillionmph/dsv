mod outputs;

use anyhow::Result as Res;
use outputs::{delimited::run_print_delimited, table::run_print_table};

use crate::cli::{OutputFormat, RunOptions};

const DEFAULT_DELIMITER: char = '\t';

pub fn run(filename_option: &Option<String>, passed_delimiter: &Option<char>, run_options: &RunOptions) -> Res<()> {
	let delimiter = get_delimiter(filename_option, passed_delimiter)?;

	match run_options.output_format {
		OutputFormat::Table {has_header, include_header_indexes} => {
			run_print_table(filename_option, delimiter, run_options, has_header, include_header_indexes)
		}
		OutputFormat::Delimited(d) => {
			let output_delimiter: u8 = d.try_into()?;
			run_print_delimited(filename_option, delimiter, output_delimiter, run_options)
		}
	}
}

fn get_delimiter(filename_option: &Option<String>, passed_delimiter: &Option<char>) -> Res<u8> {
	if let Some(d) = passed_delimiter {
		return Ok((*d).try_into()?);
	}

	if let Some(filename) = filename_option {
		let upper_case = filename.to_uppercase();
		let file_endings = [
			(".CSV", ','),
			(".TSV", '\t'),
			(".PSV", '|'),
		];

		for file_ending in file_endings {
			if upper_case.ends_with(file_ending.0) {
				return Ok(file_ending.1.try_into()?);
			}
		}
	}

	return Ok(DEFAULT_DELIMITER.try_into()?);
}
