# lingokit の組み込みメッセージ。利用側の FTL に同じキーを書けば上書きできる。

cli-arg-lang = Display language (e.g. en, ja)
cli-arg-help = Print help
cli-arg-help-long = Print help (see a summary with '-h')
cli-arg-version = Print version
cli-help-about = Print this message or the help of the given subcommand(s)
cli-help-arg-subcommand = Print help for the subcommand(s)

cli-heading-usage = Usage
cli-heading-arguments = Arguments
cli-heading-options = Options
cli-heading-commands = Commands
cli-label-possible-values = possible values
cli-label-default = default

cli-error-label = error
cli-error-hint = For more information, try '--help'.
cli-error-invalid-value = invalid value '{ $value }' for '{ $arg }' (possible values: { $valid_values })
cli-error-unknown-argument = unexpected argument '{ $arg }' found
cli-error-missing-required-argument = the following required arguments were not provided: { $arg }
cli-error-invalid-subcommand = unrecognized subcommand '{ $subcommand }'
cli-error-argument-conflict = the argument '{ $arg }' cannot be used here
cli-error-no-equals = equal sign is needed when assigning values to '{ $arg }'
cli-error-value-validation = invalid value '{ $value }' for '{ $arg }'
cli-error-too-many-values = unexpected value '{ $value }' for '{ $arg }' found
