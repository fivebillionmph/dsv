use std::{io::{BufReader, Read}, vec::IntoIter};

use csv::{Reader, StringRecord, StringRecordsIntoIter};
use anyhow::Result as Res;

pub struct ReadIterator<R: Read> {
	data_iterator: DataIterator<R>,
}

impl<'a, R: Read> ReadIterator<R> {
	pub fn new_from_vec(data: Vec<StringRecord>) -> Self {
		Self {
			data_iterator: DataIterator::Vec(data.into_iter()),
		}
	}

	pub fn new_from_csv_reader(reader: Reader<BufReader<R>>) -> Self {
		Self {
			data_iterator: DataIterator::CsvReader(reader.into_records()),
		}
	}
}

impl<R: Read> Iterator for ReadIterator<R> {
	type Item = Res<StringRecord>;

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.data_iterator {
			DataIterator::Vec(i) => {
				let next = i.next();
				match next {
					Some(n) => Some(Ok(n.clone())),
					None => None,
				}
			}
			DataIterator::CsvReader(i) => {
				let next = i.next();
				match next {
					Some(n) => {
						match n {
							Ok(r) => Some(Ok(r)),
							Err(e) => Some(Err(e.into())),
						}
					}
					None => None,
				}
			}
		}
	}
}

enum DataIterator<R: Read> {
	Vec(IntoIter<StringRecord>),
	CsvReader(StringRecordsIntoIter<BufReader<R>>),
}