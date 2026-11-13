use std::process::ExitCode;

fn main() -> ExitCode {
    // Handles --lang and prints help and clap errors localized
    let (cli, loc) = lingokit::clap::parse::<cli::Cli>(&core_lib::LOCALES);
    lingokit::clap::exit_code(&loc, cli::run(cli, &loc))
}
