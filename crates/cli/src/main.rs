use clap::{Parser, Subcommand};
use util::Executable;

mod build;
mod new;
mod util;

#[derive(Parser)]
#[clap(name = "napi", bin_name = "napi", version, about, long_about = None)]
struct Cli {
  #[clap(subcommand)]
  command: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
  New(new::NewCommand),
  Build(build::BuildCommand),
}

macro_rules! run_command {
  ( $cmd:expr, $( $name:ident ),* ) => {
    match $cmd {
      $(
        SubCommand::$name(mut sub_command) => {
          sub_command.execute().unwrap_or_else(|()| {
            std::process::exit(1);
          });
        }
      ),*
      #[allow(unreachable_patterns)]
      _ => unreachable!(),
    }
  };
}

fn main() {
  let cli = Cli::parse();

  run_command!(cli.command, New, Build);
}
