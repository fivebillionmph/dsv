use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name="dsv", about = "A utility for parsing delimited files")]
pub struct Cli {
	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
	Cat {
		filename: Option<String>,
  
		#[arg(short = 'd')]
		delimiter: Option<char>,

		#[arg(long)]
		no_header: bool,
	},
}