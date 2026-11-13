#[test]
fn cli_keys() {
    lingokit::clap::assert_keys::<cli::Cli>(&core_lib::LOCALES);
}
