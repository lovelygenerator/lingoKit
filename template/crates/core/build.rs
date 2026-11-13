fn main() {
    // locales/ の FTL を検証し、LOCALES と msg モジュールを生成する
    lingokit_build::Config::new("../../locales").env_prefix("{{env_prefix}}").generate();
}
