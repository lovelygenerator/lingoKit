// Without -d env_prefix, use crate_name in upper case (my_tool → MY_TOOL)
if !variable::is_set("env_prefix") {
    let name = variable::get("crate_name");
    name.make_upper();
    variable::set("env_prefix", name);
}
