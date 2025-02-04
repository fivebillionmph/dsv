use std::collections::HashMap;

use anyhow::{Context, Result as Res};
use csv::StringRecord;
use regex::Regex;

use crate::error::AppError;

pub struct FieldsSubset {
	field_kind: FieldKind,
}

impl FieldsSubset {
	pub fn new(numbered_fields_raw: &Option<String>, named_fields_raw: &Option<String>) -> Res<Self> {
		let field_kind = match (numbered_fields_raw, named_fields_raw) {
			(Some(_), Some(_)) => Err(AppError::new("Cannot use numbered fields and named fields together").into()),
			(Some(n), None) => Self::parse_numbered_fields(n),
			(None, Some(n)) => Self::parse_named_fields(n),
			(None, None) => Ok(FieldKind::None),
		};
		let field_kind = field_kind?;

		Ok(Self {
			field_kind,
		})
	}

	pub fn generate_file_data(&self) -> FileData {
		let mut file_data = FileData::default();
		match &self.field_kind {
			FieldKind::Named(_) => {}
			FieldKind::Numbered(v) => {
				let mut max = 0;
				for c in v {
					max = usize::max(max, *c);
				}
				file_data.max_index_required = max;
				file_data.indexes = v.clone();
			}
			FieldKind::None => {
				file_data.complete = true;
			}
		}

		file_data
	}

	pub fn set_from_row(&self, file_data: &mut FileData, row: &StringRecord, is_first_row: bool) -> Res<()> {
		if file_data.complete == true {
			return Ok(());
		}

		// For named columns
		if let Some(named_vec) = self.field_kind.get_named_vec() {
			if is_first_row {
				let mut col_to_index = HashMap::new();
				for (index, col) in row.iter().enumerate() {
					if col_to_index.contains_key(col) {
						continue;
					}
					col_to_index.insert(col.to_string(), index);
				}

				let mut missing_columns = Vec::new();
				for nv_col in named_vec {
					if let Some(col_index) = col_to_index.get(nv_col) {
						file_data.indexes.push(*col_index);
					} else {
						missing_columns.push(nv_col);
					}
				}

				if !missing_columns.is_empty() {
					return Err(AppError::new(&format!("Missing columns in named fields: {}", missing_columns.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "))).into());
				}

				file_data.complete = true;
				file_data.max_index_required = file_data.indexes.iter().fold(0, |acc, curr| acc.max(*curr));
				return Ok(()); // Nothing left to do for named columns
			}
		}

		if self.field_kind.is_numbered_vec() {
			file_data.max_index_seen = file_data.max_index_seen.max(row.len() - 1);
			if file_data.max_index_seen >= file_data.max_index_required {
				file_data.complete = true;
			}
		}

		Ok(())
	}

	pub fn transform_row(&self, file_data: &FileData, row: StringRecord) -> StringRecord {
		if self.field_kind.is_none() {
			return row;
		}

		let mut new_row = StringRecord::with_capacity(file_data.indexes.len(), file_data.indexes.len());
		for index in &file_data.indexes {
			if *index > file_data.max_index_seen {
				continue;
			}
			if let Some(c) = row.get(*index) {
				new_row.push_field(c);
			} else {
				new_row.push_field("");
			}
		}

		new_row
	}

	pub fn transform_col_widths(&self, file_data: &FileData, row: Vec<usize>) -> Vec<usize> {
		if self.field_kind.is_none() {
			return row;
		}

		let mut new_row = Vec::with_capacity(file_data.indexes.len());
		for index in &file_data.indexes {
			if *index > file_data.max_index_seen {
				continue;
			}
			if let Some(c) = row.get(*index) {
				new_row.push(*c);
			} else {
				new_row.push(0);
			}
		}

		new_row
	}

	fn parse_numbered_fields(raw: &str) -> Res<FieldKind> {
		let number_regex = Regex::new(r"^\d+$")?;
		let range_regex = Regex::new(r"^(\d+)-(\d+)$")?;
		let mut fields = vec![];
		for entry in raw.split(",") {
			if number_regex.is_match(entry) {
				let entry_number: usize = entry.parse()?;
				if entry_number < 1 {
					return Err(AppError::new("Fields are 1-indexed.  Cannot provide a 0.").into());
				}
				let entry_number = entry_number - 1; // Convert to 0-indexing
				fields.push(entry_number);
			} else if range_regex.is_match(entry) {
				let split_items = entry.split_once("-").context("Invalid range entry")?;
				let range_start: usize = split_items.0.parse()?;
				let range_end: usize = split_items.1.parse()?;
				if range_start < 1 || range_end < 1 {
					return Err(AppError::new("Fields are 1-indexed.  Cannot provide a 0.").into());
				}
				let range_start = range_start - 1; // Convert to 0-indexing
				let range_end = range_end - 1; // Convert to 0-indexing
				if range_start >= range_end {
					return Err(AppError::new(&format!("Invalid range {entry}, start must be less than end")).into());
				}
				if range_end - range_start > 10_000 {
					return Err(AppError::new("Range is too high").into()); // Sanity check
				}

				for i in range_start..=range_end {
					fields.push(i);
				}
			}
		}

		Ok(FieldKind::Numbered(fields))
	}

	fn parse_named_fields(raw: &str) -> Res<FieldKind> {
		let mut fields = vec![];
		for entry in raw.split(",") {
			fields.push(entry.into());
		}
		Ok(FieldKind::Named(fields))
	}
}

pub enum FieldKind {
	None,
	Numbered(Vec<usize>),
	Named(Vec<String>)
}
impl FieldKind {
	fn get_named_vec(&self) -> Option<&Vec<String>> {
		match self {
			Self::Named(v) => Some(v),
			_ => None,
		}
	}

	fn is_numbered_vec(&self) -> bool {
		match self {
			Self::Numbered(_) => true,
			_ => false,
		}
	}

	fn is_none(&self) -> bool {
		match self {
			Self::None => true,
			_ => false,
		}
	}
}

pub struct FileData {
	indexes: Vec<usize>,
	max_index_seen: usize,
	max_index_required: usize,
	complete: bool,
}
impl Default for FileData {
	fn default() -> Self {
		Self {
			indexes: Vec::new(),
			max_index_seen: 0,
			max_index_required: 0,
			complete: false,
		}
	}
}