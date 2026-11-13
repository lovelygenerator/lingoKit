# CLI のキー規約: cli-about / cli-<path>-about / cli-<path>-arg-<id> / cli-<path>-arg-<id>-value-<v>
# 見出し・help/version/lang 引数・clap エラーは lingokit の組み込み訳を使う(ここに書けば上書き)

cli-about = Sample CLI

cli-count-about = Count words in a text file
cli-count-arg-path = Path to the text file
cli-count-arg-format = Output format
cli-count-arg-format-value-text = Human readable text
cli-count-arg-format-value-json = JSON object
