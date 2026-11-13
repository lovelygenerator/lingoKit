// env_prefix が未指定なら crate_name を大文字にして使う(例: my_tool → MY_TOOL)
if !variable::is_set("env_prefix") {
    let name = variable::get("crate_name");
    name.make_upper();
    variable::set("env_prefix", name);
}
