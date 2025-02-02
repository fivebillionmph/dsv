use anyhow::{Context, Result as Res};
use regex::Regex;

use crate::error::AppError;

pub enum FieldsSubset {
	None,
	Numbered(Vec<usize>),
	Named(Vec<String>)
}

impl FieldsSubset {
	pub fn new(numbered_fields_raw: &Option<String>, named_fields_raw: &Option<String>) -> Res<Self> {
		if numbered_fields_raw.is_none() && named_fields_raw.is_none() {
			return Ok(Self::None);
		}

		if numbered_fields_raw.is_some() && named_fields_raw.is_some() {
		}

		match (numbered_fields_raw, named_fields_raw) {
			(Some(_), Some(_)) => Err(AppError::new("Cannot use numbered fields and named fields together").into()),
			(Some(n), None) => Self::parse_numbered_fields(n),
			(None, Some(n)) => Self::parse_named_fields(n),
			(None, None) => Ok(FieldsSubset::None),
		}
	}

	fn parse_numbered_fields(raw: &str) -> Res<Self> {
		let number_regex = Regex::new(r"^\d+$")?;
		let range_regex = Regex::new(r"^(\d+)-(\d+)$")?;
		let mut fields = vec![];
		for entry in raw.split(",") {
			if number_regex.is_match(entry) {
				let entry_number: usize = entry.parse()?;
				fields.push(entry_number);
			} else if range_regex.is_match(entry) {
				let split_items = entry.split_once("-").context("Invalid range entry")?;
				let range_start: usize = split_items.0.parse()?;
				let range_end: usize = split_items.1.parse()?;
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

		Ok(Self::Numbered(fields))
	}

	fn parse_named_fields(raw: &str) -> Res<Self> {
		let mut fields = vec![];
		for entry in raw.split(",") {
			fields.push(entry.into());
		}
		Ok(Self::Named(fields))
	}
}