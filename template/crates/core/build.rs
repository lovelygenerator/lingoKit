fn main() {
    // Checks the FTL files in locales/ and generates LOCALES and the msg module
    lingokit_build::Config::new("../../locales").env_prefix("{{env_prefix}}").generate();
}
