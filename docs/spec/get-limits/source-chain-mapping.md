# Source Chain Interface Mapping

| Interface mode | Source chain |
| --- | --- |
| Terminal default limits output | `fast_free` |
| Terminal `--best` / `-b` | `cli_fallback` |
| Desktop | `cli_first` |

`--all` is diagnostic: it queries every current source separately and does not apply source chains.
