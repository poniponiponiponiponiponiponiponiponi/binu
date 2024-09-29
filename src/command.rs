use std::path::{Path, PathBuf};

use clap::{ArgGroup, Args, Parser, Subcommand};

use binu::{GrepConfig, InsertConfig, ReplaceConfig};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Quiet output
    #[arg(short, long)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Grep-like subcommand
    #[clap(visible_alias("g"))]
    Grep(GrepArgs),

    /// Search and replace on the matches
    #[clap(visible_alias("r"))]
    Replace(ReplaceArgs),

    /// Insert bytes at the given offset
    #[clap(visible_alias("i"))]
    Insert(InsertArgs),
}

#[derive(Debug, Args)]
pub struct GrepArgs {
    /// When a directory is provided, recursively operate on all the files
    /// and subdirectories.
    #[arg(short, long)]
    pub recursive: bool,
    
    /// Pattern to search for
    pub pattern: String,

    /// Files to search for
    #[clap(required = true, num_args = 1..)]
    pub filenames: Vec<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ReplaceArgs {
    /// Pattern to replace
    pub pattern: String,

    /// Replacing string
    pub replace_with: String,

    /// When replacing only one, which occurrence to replace, counting from 0
    #[arg(short, long, default_value_t = 0)]
    pub nth: usize,

    /// Replace all occurences
    #[arg(long)]
    pub replace_all: bool,

    /// Allow using longer replace strings than the matched patterns.
    /// Warning! Will result in a changed output binary size so it may
    /// cause changing of offsets, making some binary formats unreadable
    #[arg(long)]
    pub allow_length_change: bool,

    /// When the replacing byte string is shorter than the replaced ones,
    /// fill the rest with this byte
    #[arg(long, default_value_t = 0)]
    pub fill_byte: u8,

    /// File to replace
    #[clap(required = true)]
    pub input_filename: PathBuf,

    /// Output file
    #[clap(required = true)]
    pub output_filename: PathBuf,
}

#[derive(Debug, Args)]
pub struct InsertArgs {
    /// What to insert
    pub to_insert: String,
    
    /// At what offset. Starting from 0
    pub offset: usize,

    /// To which file to insert
    pub input_filename: PathBuf,

    /// Output file
    #[clap(required = true)]
    pub output_filename: PathBuf,
}

impl Cli {
    pub fn exec(&self) {
        match &self.command {
            Commands::Grep(grep_args) => {
                let grep_config = GrepConfig {
                    quiet: self.quiet,
                    recursive: grep_args.recursive,
                };
                binu::grep_command(
                    grep_args.pattern.as_bytes(),
                    &grep_args.filenames,
                    &grep_config,
                ).unwrap_or_else(|e| {
                    eprintln!("Grep encountered error: {}", e);
                });
            }
            Commands::Replace(replace_args) => {
                let replace_config = ReplaceConfig {
                    quiet: self.quiet,
                    nth: replace_args.nth,
                    replace_all: replace_args.replace_all,
                    fill_byte: replace_args.fill_byte,
                    allow_length_change: replace_args.allow_length_change,
                };
                binu::replace_command(
                    replace_args.pattern.as_bytes(),
                    replace_args.replace_with.as_bytes(),
                    &replace_args.input_filename,
                    &replace_args.output_filename,
                    &replace_config,
                ).unwrap_or_else(|e| {
                    eprintln!("Replace encountered error: {}", e);
                });
            }
            Commands::Insert(insert_args) => {
                let insert_config = InsertConfig {
                    quiet: self.quiet,
                };
                binu::insert_command(
                    insert_args.to_insert.as_bytes(),
                    insert_args.offset,
                    &insert_args.input_filename,
                    &insert_args.output_filename,
                    &insert_config,
                ).unwrap_or_else(|e| {
                    eprintln!("Insert encountered error: {}", e);
                });
            }
        }
    }
}
