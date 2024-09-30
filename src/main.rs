mod command;

use crate::command::Cli;

use clap::Parser;

fn main() {
    let cli = Cli::parse();
    cli.exec();
}
