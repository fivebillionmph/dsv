use std::{fs::{self, File}, io::{self, BufReader, Read, Write}, path::PathBuf, str::FromStr};

use anyhow::Result as Res;
use csv::{Position, Reader, StringRecord};

use crate::{cli::RunOptions, error::AppError, fields_subset::{FieldsSubset, FileData}, read_iterator::ReadIterator};

use super::{get_csv_reader, BIG_FILE_LIMIT};

pub fn run_print_table(filename_option: &Option<String>, delimiter: u8, run_options: &RunOptions, has_header: bool, include_header_indexes: bool) -> Res<()> {
	// Parse the table to get iterator and printing info
	let (mut rows_iter, col_widths, fields_file_data) = if let Some(filename) = filename_option {
		let path = PathBuf::from_str(&filename)?;
		if !path.exists() {
			return Err(AppError::new(&format!("File doesn't exist: {}", filename)).into());
		}

		let f = File::open(&path)?;
		let reader = BufReader::new(f);
		let file_size = fs::metadata(&path)?.len();
		let mut csv_reader = get_csv_reader(reader, delimiter);
		let is_big_file = file_size > BIG_FILE_LIMIT;
		let (rows, col_widths, file_data) = parse_file(&mut csv_reader, !is_big_file, &run_options.fields_subset, include_header_indexes)?;
		let iterator = if is_big_file {
			csv_reader.seek(Position::new())?;
			ReadIterator::new_from_csv_reader(csv_reader)
		} else {
			ReadIterator::new_from_vec(rows)
		};
		(iterator, col_widths, file_data)
	} else {
		let mut csv_reader = get_csv_reader(io::stdin(), delimiter);
		let (rows, col_widths, file_data) = parse_file(&mut csv_reader, true, &run_options.fields_subset, include_header_indexes)?;
		(ReadIterator::new_from_vec(rows), col_widths, file_data)
	};

	// Run through the iterator to print each row
	let mut has_data = false;
	let row_total_length = get_table_row_total_length(&col_widths);
	let col_widths = run_options.fields_subset.transform_col_widths(&fields_file_data, col_widths);
	if let Some(first_row) = rows_iter.next() {
		print_table_line_row(&col_widths, row_total_length)?;
		let first_row = run_options.fields_subset.transform_row(&fields_file_data, first_row?);
		print_table_row(&first_row, &col_widths, row_total_length, include_header_indexes)?;
		if has_header {
			print_table_line_row(&col_widths, row_total_length)?;
		} else {
			has_data = true;
		}
	}
	for row in rows_iter {
		if !has_data {
			has_data = true;
		}
		let row = run_options.fields_subset.transform_row(&fields_file_data, row?);
		print_table_row(&row, &col_widths, row_total_length, false)?;
	}

	// Only print the last line if there was data
	if has_data {
		print_table_line_row(&col_widths, row_total_length)?;
	}

	Ok(())
}

fn parse_file<R: Read>(csv_reader: &mut Reader<R>, save_rows: bool, fields_subset: &FieldsSubset, include_header_indexes: bool) -> Res<(Vec<StringRecord>, Vec<usize>, FileData)> {
	let mut col_widths = Vec::new();
	let mut rows = vec![];
	let mut is_first_row = true;
	let mut fields_file_data = fields_subset.generate_file_data();
	for row in csv_reader.records() {
		let row = row?;
		fields_subset.set_from_row(&mut fields_file_data, &row, is_first_row)?;

		while col_widths.len() < row.len() {
			col_widths.push(0);
		}
		for (i, col) in row.iter().enumerate() {
			if let Some(this_col_width) = col_widths.get_mut(i) {
				// Tabs will be converted to 4 spaces, so add 3 extra characters per tab
				let mut col_width = col.chars().count() + (col.chars().filter(|c| *c == '\t').count() * 3);
				if is_first_row && include_header_indexes {
					let col_index_str_len = ((i + 1) / 10) + 3; // Index index format: '1. '
					col_width += col_index_str_len;
				}

				if *this_col_width < col_width {
					*this_col_width = col_width;
				}
			}
		}

		if save_rows {
			rows.push(row);
		}

		if is_first_row {
			is_first_row = false;
		}
	}

	Ok((rows, col_widths, fields_file_data))
}

fn print_table_row(row: &StringRecord, col_widths: &Vec<usize>, capacity: usize, print_header_indexes: bool) -> Res<()> {
	let mut row_string = String::with_capacity(capacity);
	for (i, col_width) in col_widths.iter().enumerate() {
		row_string += "| ";
		let remaining;
		if let Some(cell) = row.get(i) {
			let mut printable_cell = cell.replace("\t", "    ");
			if print_header_indexes {
				let header_index = i + 1;
				printable_cell = format!("{header_index}. {printable_cell}");
			}
			row_string += printable_cell.as_str();
			if *col_width > 0 {
				remaining = col_width - printable_cell.chars().count();
			} else {
				remaining = 0;
			}
		} else {
			remaining = *col_width;
		}
		if remaining > 0 {
			row_string += " ".repeat(remaining).as_str();
		}
		row_string += " ";
	}
	row_string += "|";
	writeln!(&mut std::io::stdout(), "{}", row_string)?;
	Ok(())
}

fn print_table_line_row(col_widths: &Vec<usize>, capacity: usize) -> Res<()> {
	let mut row_string = String::with_capacity(capacity);
	for col_width in col_widths {
		row_string += "+";
		row_string += "-".repeat(*col_width + 2).as_str();
	}
	row_string += "+";
	writeln!(&mut std::io::stdout(), "{}", row_string)?;
	Ok(())
}

fn get_table_row_total_length(col_widths: &Vec<usize>) -> usize {
	let mut total_length = col_widths.len() + 1; // initialize with border on left and right and between each column
	for width in col_widths {
		total_length += *width + 2; // width of column + 2 for extra space on each side
	}

	total_length
}