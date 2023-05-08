mod tui;
mod types;

use argh::FromArgs;
use tui::Tui;
use types::ConfigFile;

fn main() {
    let args: Args = argh::from_env();

    match args.command {
        Some(Command::Init(_)) => todo!(),
        None => test(),
    }
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(description = "Command to run")]
struct Args {
    #[argh(subcommand)]
    command: Option<Command>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Command {
    Init(InitCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "init",
    description = "create a plain nixsec configuration file in the current folder"
)]
struct InitCommand {}

fn test() {
    let config =
        ConfigFile::from_file("/home/pete/src/gitlab.com/leaguemaster/leaguemaster/.nixsec.nix")
            .expect("Could not open file");

    // config.write();

    let mut tui = Tui::new();
    tui.run();

    // dbg!(config);
}
