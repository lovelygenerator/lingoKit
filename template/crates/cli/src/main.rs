use std::process::ExitCode;

fn main() -> ExitCode {
    // --lang の処理、help・clap エラーの翻訳まで行う
    let (cli, loc) = lingokit::clap::parse::<cli::Cli>(&core_lib::LOCALES);
    lingokit::clap::exit_code(&loc, cli::run(cli, &loc))
}
