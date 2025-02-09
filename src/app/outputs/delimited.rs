use std::{fs::File, io::{self, BufReader, Read}, path::PathBuf, str::FromStr};

use anyhow::Result as Res;
use csv::{Reader, WriterBuilder};

use crate::{app::outputs::get_csv_reader, cli::RunOptions, error::AppError, fields_subset::FieldsSubset};

pub fn run_print_delimited(filename_option: &Option<String>, input_delimiter: u8, output_delimiter: u8, run_options: &RunOptions) -> Res<()> {
	if let Some(filename) = filename_option {
		let path = PathBuf::from_str(&filename)?;
		if !path.exists() {
			return Err(AppError::new(&format!("File doesn't exist: {}", filename)).into());
		}

		let f = File::open(&path)?;
		let reader = BufReader::new(f);
		let mut csv_reader = get_csv_reader(reader, input_delimiter);
		parse_and_write_file(&mut csv_reader, output_delimiter, &run_options.fields_subset)?;
	} else {
		let mut csv_reader = get_csv_reader(io::stdin(), input_delimiter);
		parse_and_write_file(&mut csv_reader, output_delimiter, &run_options.fields_subset)?;
	};

	Ok(())
}

fn parse_and_write_file<R: Read>(csv_reader: &mut Reader<R>, output_delimiter: u8, fields_subset: &FieldsSubset) -> Res<()> {
	let mut first_row = false;
	let mut fields_file_data = fields_subset.generate_file_data();
	let mut writer = WriterBuilder::new().delimiter(output_delimiter).from_writer(io::stdout());
	for row in csv_reader.records() {
		let row = row?;
		fields_subset.set_from_row(&mut fields_file_data, &row, first_row)?;
		if first_row {
			first_row = false;
		}
		let row = fields_subset.transform_row(&fields_file_data, row);
		writer.write_record(&row)?;
	}

	Ok(())
}