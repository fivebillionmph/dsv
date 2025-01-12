use std::{fs::{self, File}, io::{BufRead, BufReader}, path::PathBuf, str::FromStr};

use anyhow::Result as Res;
use csv::StringRecord;

use crate::error::AppError;

const BIG_FILE_LIMIT: u64 = 100 * 1024 * 1024; // 100 MB
const DEFAULT_DELIMITER: char = '\t';

pub fn run(filename_option: &Option<String>, passed_delimiter: &Option<char>, no_header: bool) -> Res<()> {
	let mut delimiter = None;
	if let Some(pd) = passed_delimiter {
		delimiter = Some(pd.clone());
	}

	if let Some(filename) = filename_option {
		let path = PathBuf::from_str(&filename)?;
		if !path.exists() {
			return Err(AppError::new(&format!("File doesn't exist: {}", filename)).into());
		}

		if delimiter.is_none() {
			delimiter = get_delimiter_from_filename(&filename);
		}
		let delimiter = get_delimiter_u8(delimiter)?;

		let file_size = fs::metadata(&path)?.len();
		if file_size > BIG_FILE_LIMIT {
			parse_big_file(&path, delimiter)
		} else {
			parse_small_file(&path, delimiter, !no_header)
		}
	} else {
		let delimiter = get_delimiter_u8(delimiter)?;
		parse_stdin(delimiter)
	}
}

fn get_delimiter_from_filename(filename: &str) -> Option<char> {
	let upper_case = filename.to_uppercase();
	let file_endings = [
		(".CSV", ','),
		(".TSV", '\t'),
		(".PSV", '|'),
	];

	for file_ending in file_endings {
		if upper_case.ends_with(file_ending.0) {
			return Some(file_ending.1);
		}
	}

	None
}

fn parse_small_file(path: &PathBuf, delimiter: u8, has_header: bool) -> Res<()> {
	let f = File::open(path)?;
	let reader = BufReader::new(f);

	let mut csv_reader = csv::ReaderBuilder::new()
		.has_headers(false)
		.delimiter(delimiter)
		.flexible(true)
		.from_reader(reader);

	let mut col_widths = Vec::new();
	let mut rows = vec![];
	for result in csv_reader.records() {
		let row = result?;
		while col_widths.len() < row.len() {
			col_widths.push(0);
		}
		for (i, col) in row.iter().enumerate() {
			if let Some(this_col_width) = col_widths.get_mut(i) {
				if *this_col_width < col.len() {
					*this_col_width = col.len();
				}
			}
		}

		rows.push(row);
	}

	let row_total_length = get_row_total_length(&col_widths);
	let mut rows_iter = rows.iter();
	print_line_row(&col_widths, row_total_length);
	if has_header {
		if let Some(first_row) = rows_iter.next() {
			print_row(first_row, &col_widths, row_total_length);
		}
		print_line_row(&col_widths, row_total_length);
	}
	for row in rows_iter {
		print_row(row, &col_widths, row_total_length);
	}
	print_line_row(&col_widths, row_total_length);

	Ok(())
}

fn parse_big_file(path: &PathBuf, delimiter: u8) -> Res<()> {
	todo!()
}

fn parse_stdin(delimiter: u8) -> Res<()> {
	todo!()
}

fn get_delimiter_u8(delimiter: Option<char>) -> Res<u8> {
	let delimiter = match delimiter {
		Some(s) => s,
		None => DEFAULT_DELIMITER,
	};
	Ok(delimiter.try_into()?)
}

fn print_row(row: &StringRecord, col_widths: &Vec<usize>, capacity: usize) {
	let mut row_string = String::with_capacity(capacity);
	for (i, col_width) in col_widths.iter().enumerate() {
		row_string += "| ";
		let remaining;
		if let Some(cell) = row.get(i) {
			row_string += cell;
			if *col_width > 0 {
				remaining = col_width - cell.len();
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
	println!("{}", row_string);
}

fn print_line_row(col_widths: &Vec<usize>, capacity: usize) {
	let mut row_string = String::with_capacity(capacity);
	for col_width in col_widths {
		row_string += "+";
		row_string += "-".repeat(*col_width + 2).as_str();
	}
	row_string += "+";
	println!("{}", row_string);
}

fn get_row_total_length(col_widths: &Vec<usize>) -> usize {
	let mut total_length = col_widths.len() + 1; // initialize with border on left and right and between each column
	for width in col_widths {
		total_length += *width + 2; // width of column + 2 for extra space on each side
	}

	total_length
}