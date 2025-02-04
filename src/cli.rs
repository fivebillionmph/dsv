use clap::Parser;
use anyhow::Result as Res;
use crate::fields_subset::FieldsSubset;

#[derive(Parser, Debug)]
#[command(name="dsv", about = "A utility for parsing delimited files")]
struct Cli {
	pub filename: Option<String>,
 
	#[arg(short = 'd', help = "delimiter character (defaults to tab if can't figure out from filename)")]
	pub delimiter: Option<char>,

	#[arg(long, help = "Don't print extra lines separating the header from the filename rest of the file when outputting in table format")]
	pub no_header: bool,

	#[arg(short = 'f', help = "Print a subset of the columns based on the number index (1-indexed, comma separated)")]
	pub number_fields: Option<String>,

	#[arg(short = 'F', help = "Print a subset of the columns based on the header name (comma separated)")]
	pub named_fields: Option<String>,

	#[arg(short = 'o', help = "How should the result be printed", default_value = "table")]
	pub output_format: CliOutputFormat,

	#[arg(long, help = "Output delimiter if using delimited format (default tab)")]
	pub output_delimiter: Option<char>,
}

#[derive(Debug, Clone, Default, clap::ValueEnum)]
enum CliOutputFormat {
	#[default]
	Table,
	Delimited,
}

pub fn get_run_options() -> Res<(Option<String>, Option<char>, RunOptions)> {
	let args = Cli::parse();
	Ok((args.filename.clone(), args.delimiter.clone(), RunOptions::new(&args)?))
}

pub struct RunOptions {
	pub has_header: bool,
	pub fields_subset: FieldsSubset,
	pub output_format: OutputFormat,
}

impl RunOptions {
	fn new(args: &Cli) -> Res<Self> {
		let output_format = match &args.output_format {
			CliOutputFormat::Table => OutputFormat::Table,
			CliOutputFormat::Delimited => {
				match args.output_delimiter {
					Some(c) => OutputFormat::Delimited(c),
					None => OutputFormat::Delimited('\t'),
				}
			}
		};

		Ok(Self {
			has_header: !args.no_header,
			fields_subset: FieldsSubset::new(&args.number_fields, &args.named_fields)?,
			output_format: output_format,
		})
	}
}

pub enum OutputFormat {
	Table,
	Delimited(char),
}