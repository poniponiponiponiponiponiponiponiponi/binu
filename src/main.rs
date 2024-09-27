mod command;

use crate::command::{Cli, Commands};

use clap::Parser;

fn main() {
    let cli = Cli::parse();
    cli.exec();
}
