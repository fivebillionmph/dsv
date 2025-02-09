use std::io::Read;

use csv::Reader;

pub mod table;
pub mod delimited;

const BIG_FILE_LIMIT: u64 = 100 * 1024 * 1024; // 100 MB

fn get_csv_reader<R: Read>(reader: R, delimiter: u8) -> Reader<R> {
	csv::ReaderBuilder::new()
		.has_headers(false)
		.delimiter(delimiter)
		.flexible(true)
		.from_reader(reader)
}